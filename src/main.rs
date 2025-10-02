use clap::Parser;
// use cli::Cli;
use color_eyre::Result;

use crate::app::App;
use crate::cli::AppConfiguration;
use crate::config::AppConfig;
// use crate::config::AppConfig;
use crate::teamcity::TeamCityClient;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod tui;
mod teamcity;
mod utils;
mod time;

#[tokio::main]
async fn main() -> Result<()> {
    errors::init()?;
    logging::init()?;

    // let args = Cli::parse();
    let config = match AppConfiguration::new() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Please check your config file or environment variables.");
            std::process::exit(1);
        }
    };

    let teamcity_url = config.teamcity_url;
    let token = config.token;
    let projects = config.projects;
    let client = TeamCityClient::new(teamcity_url, token);

    println!("Fetching build configurations from TeamCity...");
    let build_types = client.get_build_configurations_by_projects(&projects).await.unwrap();

    let mut app = App::new(client, build_types)?;
    app.run().await?;
    Ok(())
}
