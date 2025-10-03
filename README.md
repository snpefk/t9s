# t9s

A terminal user interface (TUI) for browsing TeamCity build configurations.

## Features

- TeamCity integration: browse projects and build configurations from your server
- Fuzzy search for projects/builds (depends on system `fzf`)
- Project filtering to limit the scope to the projects you care about
- Open builds in your default browser right from the TUI
- Persistent on‑disk cache for projects to reduce API calls
- Vim‑style navigation and key‑driven workflow
- View build logs with your default `$PAGER` in terminal

## Getting started

Prerequisites:

- Rust and Cargo installed
- (optional) `fzf` installed (for fuzzy search)
- A TeamCity server URL and
  a [personal access token](https://www.jetbrains.com/help/teamcity/manage-access-tokens.html#Token+Management)

Build:

- `cargo build --release`

Run (first time):

- `cargo run --release`

> [!IMPORTANT]
> If required values are not provided, the app will guide you through *an interactive setup* and save a config file in
> your per‑user config directory (as determined by the OS). The app prints the exact config and data directories at
> startup.

Run (with CLI flags):

- `cargo run --release -- --teamcity-url https://teamcity.example.com --token <TOKEN> --projects PROJ1_ID,PROJ2_ID`

Environment variables (alternative to flags):

- `T9S_TEAMCITY_URL` — TeamCity server URL
- `T9S_TEAMCITY_TOKEN` — personal access token
- `T9S_TEAMCITY_PROJECTS` — comma‑separated project IDs

After launch, the app fetches build configurations for the configured projects and opens the TUI.
Use the on‑screen hints and navigation keys to explore and open builds in your browser.