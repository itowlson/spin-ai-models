use std::path::{PathBuf, Path};

use anyhow::{Context, anyhow};
use clap::Parser;

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Command
}

#[derive(Parser)]
enum Command {
    #[command()]
    Install(InstallCommand),
}

#[derive(Parser)]
struct InstallCommand {
    #[arg()]
    model_name: Option<String>,

    #[arg(
        name = "MANIFEST_FILE",
        short = 'f',
        long = "from",
        alias = "file",
        default_value = "spin.toml"
    )]
    pub app_source: PathBuf,

    #[arg(env = "SPIN_VERSION")]
    pub target_spin_version: Option<String>
}

fn main() -> anyhow::Result<()> {
    App::parse().run()
}

impl App {
    fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Install(cmd) => cmd.run(),
        }
    }
}

impl InstallCommand {
    fn run(&self) -> anyhow::Result<()> {
        let manifest_file = spin_common::paths::resolve_manifest_file_path(&self.app_source)?;
        if !manifest_file.is_file() {
            anyhow::bail!("Manifest file '{}' not found", manifest_file.display());
        }
        let manifest_dir = manifest_file.parent().unwrap();  // life is too short

        let model_names = self.model_names();
        if model_names.is_empty() {
            eprintln!("No models selected");
            return Ok(());
        }

        let models_dir = manifest_dir.join(".spin/ai-models");
        std::fs::create_dir_all(&models_dir)
            .with_context(|| format!("Failed to create models dir '{}'", models_dir.display()))?;

        download_models(&model_names, &models_dir)?;

        Ok(())
    }

    fn model_names(&self) -> Vec<String> {
        match &self.model_name {
            Some(name) => vec![name.clone()],
            None => prompt_names(),
        }
    }
}

fn prompt_names() -> Vec<String> {
    let known_models = &["llama2-chat", "codellama-instruct", "all-minikm-16-v2"];

    dialoguer::MultiSelect::new()
        .with_prompt("Select items using up/down arrow and Space. Esc to cancel, Enter to accept")
        .items(known_models)
        .interact_opt()
        .unwrap_or_else(|_| std::process::exit(0))
        .unwrap_or_else(|| std::process::exit(0))
        .into_iter()
        .map(|index| known_models[index].to_owned())
        .collect::<Vec<_>>()
}

fn download_models(model_names: &[String], model_dir: &Path) -> anyhow::Result<()> {
    for model_name in model_names {
        download_model(model_name, model_dir)?;
    }
    Ok(())
}

fn download_model(model_name: &str, model_dir: &Path) -> anyhow::Result<()> {
    if model_name == "llama2-chat" {
        let model_file = model_dir.join(model_name);
        download("https://huggingface.co/TheBloke/Llama-2-13B-chat-GGML/resolve/a17885f653039bd07ed0f8ff4ecc373abf5425fd/llama-2-13b-chat.ggmlv3.q3_K_L.bin", &model_file)?;
    } else if model_name == "codellama-instruct" {
        let model_file = model_dir.join(model_name);
        download("https://huggingface.co/TheBloke/CodeLlama-13B-Instruct-GGML/resolve/b3dc9d8df8b4143ee18407169f09bc12c0ae09ef/codellama-13b-instruct.ggmlv3.Q3_K_L.bin", &model_file)?;
    } else if model_name == "all-minikm-16-v2" {
        let model_subdir = model_dir.join(model_name);
        std::fs::create_dir_all(&model_subdir)?;
        let tokeniser_file = model_subdir.join("tokenizer.json");
        let safetensor_file = model_subdir.join("model.safetensors");
        download("https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/7dbbc90392e2f80f3d3c277d6e90027e55de9125/tokenizer.json", &tokeniser_file)?;
        download("https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/0b6dc4ef7c29dba0d2e99a5db0c855c3102310d8/model.safetensors", &safetensor_file)?;
    } else {
        anyhow::bail!("Unknown model {model_name}");
    }
    Ok(())
}

fn download(url: &str, file: &Path) -> anyhow::Result<()> {
    let (cache_path, size) = get_cache_path(url)?;

    if !cache_path.is_file() {
        let size_info = match size {
            None => "(unknown)".to_owned(),
            Some(s) => format!("{s}"),
        };
        eprintln!("Model from {url} is not in cache. Downloading {size_info} bytes to cache at {} - this may take a long time.", cache_path.display());

        let mut resp = reqwest::blocking::get(url)
            .with_context(|| format!("Making request to {url}"))?;
        if !resp.status().is_success() {
            anyhow::bail!("HTTP code {} downloading from {url}", resp.status());
        }
    
        std::fs::create_dir_all(cache_path.parent().unwrap())?;
        let mut stm = std::fs::File::create(&cache_path)?;
        resp.copy_to(&mut stm)?;
    
        eprintln!("Cached model at {}", cache_path.display());
    }

    std::fs::hard_link(&cache_path, file)?;
    Ok(())
}

fn get_cache_path(url: &str) -> anyhow::Result<(PathBuf, Option<u64>)> {
    let client = reqwest::blocking::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let resp = client.head(url).send()?;

    let (etag_key, size_key) = if resp.status().is_redirection() {
        ("x-linked-etag", "x-linked-size")
    } else {
        ("etag", "content-length")
    };

    let etag_raw = resp.headers().get(etag_key).ok_or(anyhow!("No etag"))?.to_str()?;
    let size_raw = resp.headers().get(size_key).and_then(|v| v.to_str().ok());

    let etag = etag_raw.trim_matches('"');
    let size = size_raw.and_then(|v| v.parse::<u64>().ok());

    let cache_dir = dirs::cache_dir().ok_or(anyhow!("No cache dir"))?.join("spin/ai-models");
    let cache_path = cache_dir.join(etag);

    Ok((cache_path, size))
}
