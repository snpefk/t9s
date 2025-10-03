use crate::config::{get_config_dir, get_data_dir};
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all, read_to_string};
use std::io;
use std::io::Write;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// TeamCity server URL
    #[arg(long, env = "T9S_TEAMCITY_URL")]
    pub teamcity_url: Option<String>,

    /// Authentication token
    #[arg(long, env = "T9S_TEAMCITY_TOKEN")]
    pub token: Option<String>,

    /// List of projects to monitor
    #[arg(short, long, env = "T9S_TEAMCITY_PROJECTS", value_delimiter = ',')]
    pub projects: Option<Vec<String>>,
}

impl Cli {
    pub fn load_cli_config() -> Result<Cli> {
        let cfg_dir = get_config_dir();
        let app_cfg = cfg_dir.join("config.toml");

        if app_cfg.exists() {
            println!("Loading config from {:?}", app_cfg);

            let content = read_to_string(app_cfg)?;

            toml::from_str::<Cli>(&content)
                .map_err(|e| eyre!("Failed to parse config: {}", e))
        } else {
            Err(eyre!("{:?} config file does not exists", app_cfg))
        }
    }

    pub fn save_cli_config(cli: &Cli) -> Result<()> {
        let cfg_dir = get_config_dir();
        create_dir_all(&cfg_dir)?;

        let path = cfg_dir.join("config.toml");
        let mut file = File::create(&path)?;
        let content = toml::to_string_pretty(cli)?;

        println!("Saving config to {:?}", cfg_dir);
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn init_config(projects: &Option<Vec<String>>) -> Result<Cli> {
        println!(
            "Initial setup: let's configure your TeamCity connection. Press Enter to accept defaults where shown."
        );

        let teamcity_url = Cli::prompt(
            "TeamCity server URL",
            Some("https://teamcity.example.com".to_string()),
        );
        let token = rpassword::prompt_password("Personal access token")?;
        let projects_input: Option<String> = match projects.clone() {
            Some(v) if !v.is_empty() => Some(v.join(",")),
            _ => None,
        };
        let projects_str = Cli::prompt("Projects (comma-separated, optional)", projects_input);
        let projects = {
            let s = projects_str.trim();
            if s.is_empty() {
                None
            } else {
                let list: Vec<String> = s
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
                if list.is_empty() { None } else { Some(list) }
            }
        };

        let args = Self {
            teamcity_url: Some(teamcity_url),
            token: Some(token),
            projects,
        };

        Cli::save_cli_config(&args)?;

        Ok(args)
    }

    fn prompt(label: &str, default: Option<String>) -> String {
        let mut stdout = io::stdout();

        match default {
            Some(ref val) => print!("{label} [{val}]: "),
            None => print!("{label}: "),
        }

        stdout.flush().expect("Can't flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("read_line failed");
        let trimmed = input.trim();

        if trimmed.is_empty() {
            default.unwrap_or_default()
        } else {
            trimmed.to_string()
        }
    }
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}
