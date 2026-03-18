![Stars][stars-shield] ![Forks][forks-shield] ![Contributors][contributors-shield] ![License][license-shield] 

## GitHub Saver

<details>
  <summary><strong>Table of Contents</strong></summary>

- **[Overview](#overview)**
- **[Usage](#usage)**
  - [Installation](#installation)
  - [Notes](#notes)
- **[Development](#development)**
  - [Project Structure](#project-structure)
  - [Dependencies](#dependencies)
  - [Project Status](#project-status)
- **[License](#license)**
- **[Contacts](#contacts)**

</details>

---

### Overview

GitHub Saver is a small TUI tool that backs up your GitHub repositories by cloning selected repositories into `./github_saves`.

---

### Usage

#### Installation
Install from GitHub:

```bash
cargo install --git https://github.com/github/github_saver --locked
```

Run:

```bash
github-saver
```

#### Notes

- You need `git` installed and available in `PATH`.
- The tool requires a GitHub API token. Do not use fine-grained tokens.
- Selected repositories are cloned into `./github_saves`. If the folder exists, it will be deleted before cloning.

---

### Development

![Languages][languages-shield] ![Build status][build-status-shield] ![Build tool][build-tool-shield]

#### Project Structure

```text
.github/             
  workflows/         
    ci.yml           # CI workflow
.gitignore           # Gitignore rules
src/                 
  main.rs            # Application entry point
Cargo.toml           # Cargo manifest
Cargo.lock           # Dependency lockfile
LICENSE              # License
README.md            # This file
```

#### Dependencies

- reqwest
- serde / serde_json
- crossterm
- anyhow
- unicode-width

#### Project Status

- Status: the latest stable version has been created

---

### License

This project is distributed under the **[MIT](LICENSE)** license.  

---

### Contacts

- **Author**: [MrachniyTipchek](https://github.com/MrachniyTipchek)

[stars-shield]: https://img.shields.io/github/stars/MrachniyTipchek/README-Template?style=for-the-badge
[forks-shield]: https://img.shields.io/github/forks/MrachniyTipchek/README-Template?style=for-the-badge
[contributors-shield]: https://img.shields.io/github/contributors/MrachniyTipchek/README-Template?style=for-the-badge
[license-shield]: https://img.shields.io/github/license/MrachniyTipchek/README-Template?style=for-the-badge
[languages-shield]: https://img.shields.io/badge/Languages-Rust-blue?style=for-the-badge
[build-status-shield]: https://img.shields.io/github/actions/workflow/status/MrachniyTipchek/README-Template/WORKFLOW_FILE.yml?style=for-the-badge&label=Build%20status
[build-tool-shield]: https://img.shields.io/badge/Build%20tool-Cargo-blueviolet?style=for-the-badge