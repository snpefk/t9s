mod teamcity;

use crate::teamcity::types::BuildType;
use crate::teamcity::TeamCityClient;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::crossterm::{event, execute};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table, TableState};
use ratatui::{Frame, Terminal};
use std::error::Error;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
enum AppMessage {
    None,
    Success(String),
    Error(String),
}

struct AppState {
    table_state: TableState,
    build_configs: Vec<BuildType>,
    message: AppMessage,
}

impl AppState {
    fn new(build_configs: Vec<BuildType>) -> Self {
        let mut table_state = TableState::default();
        if !build_configs.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            table_state,
            build_configs,
            message: AppMessage::None,
        }
    }

    fn move_down(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.build_configs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn move_up(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.build_configs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn open_selected_url(&mut self) {
        if let Some(selected_index) = self.table_state.selected() {
            if let Some(config) = self.build_configs.get(selected_index) {
                if let Some(web_url) = &config.web_url {
                    match open::that(web_url) {
                        Ok(_) => {
                            self.message = AppMessage::Success(
                                format!("Opened {} in browser", config.name)
                            );
                        }
                        Err(e) => {
                            self.message = AppMessage::Error(
                                format!("Failed to open URL: {}", e)
                            );
                        }
                    }
                } else {
                    self.message = AppMessage::Error(
                        "No web URL available for this build configuration".to_string()
                    );
                }
            }
        }
    }

    fn clear_message(&mut self) {
        self.message = AppMessage::None;
    }
}

fn render_ui(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    draw_main_table(frame, chunks[0], &app_state.build_configs, &mut app_state.table_state.clone());

    // Draw status message
    match &app_state.message {
        AppMessage::Success(msg) => {
            let status_block = Block::default()
                .borders(Borders::TOP)
                .title("Status")
                .style(Style::default().fg(Color::Green));
            frame.render_widget(status_block, chunks[1]);

            let text = ratatui::widgets::Paragraph::new(msg.as_str())
                .style(Style::default().fg(Color::Green));
            frame.render_widget(text, chunks[1]);
        }
        AppMessage::Error(msg) => {
            let status_block = Block::default()
                .borders(Borders::TOP)
                .title("Error")
                .style(Style::default().fg(Color::Red));
            frame.render_widget(status_block, chunks[1]);

            let text = ratatui::widgets::Paragraph::new(msg.as_str())
                .style(Style::default().fg(Color::Red));
            frame.render_widget(text, chunks[1]);
        }
        AppMessage::None => {}
    }
}

fn draw_main_table(frame: &mut Frame, area: Rect, build_configs: &[BuildType], table_state: &mut TableState) {
    let header = Row::new(vec!["Project", "Name", "ID"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> = build_configs
        .iter()
        .map(|config| {
            Row::new(vec![
                config.project_name.clone().unwrap_or_else(|| "N/A".to_string()),
                config.name.clone(),
                config.id.clone(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Max(30),
            Constraint::Min(70),
            Constraint::Max(30),
        ])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Build Configurations")
        )
        .column_spacing(1)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, table_state);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub teamcity_url: String,
    pub token: String,
    pub projects: Vec<String>,
}

impl AppConfig {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = match AppConfig::new() {
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
    let build_configs = client.get_build_configurations_by_projects(&projects).await?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut table_state = TableState::default();
    if !build_configs.is_empty() {
        table_state.select(Some(0));
    }

    let mut app_state = AppState::new(build_configs);

    loop {
        terminal.draw(|frame| {
            render_ui(frame, frame.size(), &app_state);
        })?;

        if let Event::Key(key) = event::read()? {
            // Clear message on any key press
            if !matches!(app_state.message, AppMessage::None) {
                app_state.clear_message();
            }

            match key.code {
                KeyCode::Char('q') => break,
                // TODO: add gg and G keybindings
                KeyCode::Down| KeyCode::Char('j') => {
                    app_state.move_down();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app_state.move_up();
                }
                KeyCode::Char('o') => {
                    app_state.open_selected_url();
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
