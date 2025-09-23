use clap::Parser;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use crate::config::{get_config_dir, get_data_dir, AppConfig};

// #[derive(Parser, Debug)]
// pub struct Cli {
//
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfiguration {
    pub teamcity_url: String,
    pub token: String,
    pub projects: Vec<String>,
}

impl AppConfiguration {
    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = dirs::config_dir().unwrap().join("teamcity-cli/config.toml");
        let config_builder = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(File::from(config_dir).required(false))
            .add_source(Environment::with_prefix("T9S_TEAMCITY_URL"));

        let settings = config_builder.build()?;
        settings.try_deserialize()
    }
}

// const VERSION_MESSAGE: &str = concat!(
//     env!("CARGO_PKG_VERSION"),
//     "-",
//     env!("VERGEN_GIT_DESCRIBE"),
//     " (",
//     env!("VERGEN_BUILD_DATE"),
//     ")"
// );
//
// pub fn version() -> String {
//     let author = clap::crate_authors!();
//
//     // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
//     let config_dir_path = get_config_dir().display().to_string();
//     let data_dir_path = get_data_dir().display().to_string();
//
//     format!(
//         "\
// {VERSION_MESSAGE}
//
// Authors: {author}
//
// Config directory: {config_dir_path}
// Data directory: {data_dir_path}"
//     )
// }
