# spin ai-models

A Spin plugin for downloading LLMs and installing them in the right place in your Spin app directory.

* Automates the instructions from https://developer.fermyon.com/spin/serverless-ai-tutorial#application-structure
* Caches downloads so that future installs can be done instantly

## Installation

For now:

```
cargo build --release
spin pluginify --install
```

## Usage

`spin ai-models install`

Options:

* Name of the model (`llama2-chat`, `codellama-instruct`, or `all-minikm-16-v2`). If omitted it will prompt you (and allow multi select)
* `-f` the app to install the models into

## Known issues

Honestly most of the plugin is issues with just a thin thread of stuff that works, but the big one, the really big one is that _it does not parallelise the download of these enormous great files_.  Another thing I would like to do is add the declarations to `spin.toml`, at least for the easy case where there's only one component.

## If you liked this

The `spin cloud-gpu` plugin (https://github.com/fermyon/spin-cloud-gpu) lets you run supported models on Fermyon Cloud even while running your app locally. GPU acceleration, no downloading 7GB files, what's not to love?
