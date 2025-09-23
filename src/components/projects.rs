use super::Component;
use crate::teamcity::types::BuildType;
use crate::{action::Action, config::Config};
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table, TableState};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct Projects {
    build_configs: Vec<BuildType>,
    table_state: TableState,
    // buffer to hold KeyEvents for multi-key combinations
    last_events: Vec<KeyEvent>,
}

impl Projects {
    pub fn new(build_configs: Vec<BuildType>) -> Self {
        Self {
            build_configs,
            ..Self::default()
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

    fn move_end(&mut self) {
        (self.table_state.select_last())
    }

    fn move_begin(&mut self) {
        (self.table_state.select_first())
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
                            // self.message = AppMessage::Success(
                            //     format!("Opened {} in browser", config.name)
                            // );
                        }
                        Err(e) => {
                            // self.message = AppMessage::Error(
                            //     format!("Failed to open URL: {}", e)
                            // );
                        }
                    }
                } else {
                    // self.message = AppMessage::Error(
                    //     "No web URL available for this build configuration".to_string()
                    // );
                }
            }
        }
    }
}

impl Component for Projects {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        self.last_events.push(key);

        let action = match key.code {
            KeyCode::Char('G') => {
                if key.modifiers == KeyModifiers::SHIFT {
                    self.move_end()
                }
                Action::Render
            }
            KeyCode::Char('g') => {
                if let Some(previous_key) = self.last_events.iter().rev().nth(1) {
                    if previous_key.code == KeyCode::Char('g') {
                        self.move_begin();
                        self.last_events.clear()
                    }
                }
                Action::Render
            }
            KeyCode::Char('j') => {
                self.move_down();
                Action::Render
            }
            KeyCode::Char('k') => {
                self.move_up();
                Action::Render
            }
            KeyCode::Char('f') => {
                let build_types: Vec<String> = self
                    .build_configs
                    .iter()
                    .map(|build_type: &BuildType| {
                        format!("{name} ({id})", name = build_type.name, id = build_type.id)
                    })
                    .collect();

                Action::Fzf(build_types)
            }
            _ => Action::Render,
        };
        Ok(Some(action))
    }

    fn init(&mut self, area: Size) -> color_eyre::Result<()> {
        if !self.build_configs.is_empty() {
            self.table_state.select(Some(0));
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let header = Row::new(vec!["Project", "Name", "ID"])
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .height(1)
            .bottom_margin(1);

        let rows: Vec<Row> = self
            .build_configs
            .iter()
            .map(|config| {
                Row::new(vec![
                    config
                        .project_name
                        .clone()
                        .unwrap_or_else(|| "N/A".to_string()),
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
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Build Configurations"),
        )
        .column_spacing(1)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);
        Ok(())
    }
}
