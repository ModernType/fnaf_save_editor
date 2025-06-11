# Modern FNaF Save Editor

This program allows you to edit save files of various FNaF games (Currently only *FNaF World* and *FNaF World: Refreshed*, all other are coming soon).

## Installation
Download the latest executable from `Releases` tab.

## Found a bug? Have a suggestion?
If you find any bugs or have a suggestion for new functions, please create an issue in `Issues` tab. If you want to help developing the app, you can contribute by making a *pull request*.

## Compiling
### Requirements
You need the latest version of [VS Build Tools 2022](https://visualstudio.microsoft.com/downloads/) installed and [Rust compiler toolchain](https://www.rust-lang.org/).

### Getting the project and compiling
To get the source code you can either download a .zip in `Code` tab or use `git`:
```bash
git clone https://github.com/ModernType/fnaf_save_editor.git
```
After cloning the repo open terminal in the project root folder and build the project using:
```bash
cargo build --release
```
(Build process will take quite some time)

You will find executable in `{project_folder_name}/target/release/fnaf_save_editor.exe`

### Compiling without animatronic animations
To reduce binary size (in half) and app startup time you can compile the project without animatronic animations. To do so use the following command to build the project:
```bash
cargo build --release --features no-animation
```

## Licensing
This software is licensed under GPL-3.0 license.