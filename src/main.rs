use crate::app::App;
use crate::cli::Cli;
use crate::teamcity::TeamCityClient;
use clap::Parser;
use color_eyre::Result;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod teamcity;
mod time;
mod tui;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    errors::init()?;
    logging::init()?;

    let mut args = Cli::parse();

    // Try to load args from a config file
    if args.teamcity_url.is_none() || args.token.is_none() {
        match Cli::load_cli_config() {
            Ok(loaded) => {
                if args.teamcity_url.is_none() {
                    args.teamcity_url = loaded.teamcity_url;
                }
                if args.token.is_none() {
                    args.token = loaded.token;
                }
                if args.projects.is_none() {
                    args.projects = loaded.projects;
                }
            }
            Err(e) => {
                // Consider that config file is missing and this is the first time the app is run
                eprintln!("Warning: failed to load config: {e}");
                args = Cli::init_config(&args.projects)?;
            }
        }
    }

    let teamcity_url = args.teamcity_url.expect("Something went wrong and teamcity_url parameter wasn't set");
    let token = args.token.expect("Somethings went wrong and token parameter wasn't set");
    let projects = args.projects.unwrap_or_default();

    let client = TeamCityClient::new(teamcity_url, token);

    println!("Fetching build configurations from TeamCity...");
    let build_types = client.get_build_configurations_by_projects(&projects).await?;

    let mut app = App::new(client, build_types)?;
    app.run().await?;
    Ok(())
}
