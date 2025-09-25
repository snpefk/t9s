use super::Component;
use crate::teamcity::types::BuildType;
use crate::{action::Action, config::Config};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState, Wrap};
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct Projects {
    build_types: Vec<BuildType>,
    table_state: TableState,
    input_mode: InputMode,
    input_buffer: String,
    // buffer to hold KeyEvents for multi-key combinations
    last_events: Vec<KeyEvent>,
    pub filter_string: Option<String>,
    pub action_tx: Option<UnboundedSender<Action>>,
}

#[derive(Default, PartialEq, Clone, Debug)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

impl Projects {
    pub fn new(build_configs: Vec<BuildType>) -> Self {
        Self {
            build_types: build_configs,
            filter_string: None,
            ..Self::default()
        }
    }
    
    fn get_build_types(&mut self) -> Vec<BuildType> {
        self
            .build_types
            .iter()
            .filter(|build_type| {
                if let Some(filter_string) = &self.filter_string {
                    build_type.name.to_lowercase().contains(filter_string)
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    fn filter_build_types(&mut self, filter_string: &String) {
        self.filter_string = Some(filter_string.to_string().to_lowercase());
    }

    fn move_down(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.get_build_types().len() - 1 {
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
        let n = self.get_build_types().len() - 1;
        self.table_state.select(Some(n))
    }

    fn move_begin(&mut self) {
        self.table_state.select_first()
    }

    fn move_up(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.get_build_types().len() - 1
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
            if let Some(config) = self.get_build_types().get(selected_index) {
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

    fn select_project(&mut self, selected_string: String) -> color_eyre::Result<()> {
        if let Some((i, _selected_type)) =
            self.get_build_types().iter().enumerate().find(|(_, build_type)| {
                let search_string =
                    format!("{name} ({id})", name = build_type.name, id = build_type.id);
                search_string == selected_string
            })
        {
            self.table_state.select(Some(i));
        }
        Ok(())
    }
}
pub trait ProjectsUiExt {
    fn render_input_popup(&self, frame: &mut Frame, area: Rect);
}

impl ProjectsUiExt for Projects {
    fn render_input_popup(&self, frame: &mut Frame, area: Rect) {
        let popup_width = 70;
        let popup_height = 3;

        let popup_x = area.x + ((area.width.saturating_sub(popup_width)) / 2);
        let popup_y = area.y + ((area.height.saturating_sub(popup_height)) / 2);

        let input_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width.min(area.width),
            height: popup_height.min(area.height),
        };

        let input = Paragraph::new(self.input_buffer.as_ref() as &str)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .block(
                Block::default()
                    .title("Filter build types (press Enter to apply, Esc to cancel)")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(Clear, input_area);
        frame.render_widget(input, input_area);
        frame.set_cursor_position(
            (input_area.x + self.input_buffer.len() as u16 + 1, input_area.y + 1)
        );
    }
}

impl Component for Projects {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
        Ok(())
    }

    fn init(&mut self, _area: Size) -> color_eyre::Result<()> {
        if !self.build_types.is_empty() {
            self.table_state.select(Some(0));
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        self.last_events.push(key);

        let action= if self.input_mode == InputMode::Normal {
             match key.code {
                KeyCode::Char('G') => {
                    self.move_end();
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
                        .get_build_types()
                        .iter()
                        .map(|build_type: &BuildType| {
                            format!("{name} ({id})", name = build_type.name, id = build_type.id)
                        })
                        .collect();

                    Action::Fzf(build_types)
                }
                KeyCode::Char('o') => {
                    self.open_selected_url();
                    Action::Render
                }
                KeyCode::Char('/') => {
                    self.input_mode = InputMode::Editing;
                    Action::Render
                }
                _ => Action::Render,
            }
        } else {
            match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    Action::Render
                }
                KeyCode::Char(c) if c.is_alphanumeric() || c.is_ascii_graphic() || c == ' ' => {
                    self.input_buffer.push(c);
                    Action::Render
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                    Action::Render
                }
                KeyCode::Enter => {
                    let buffer_clone = self.input_buffer.clone();
                    self.filter_build_types(&buffer_clone);
                    self.input_buffer.clear();
                    self.input_mode = InputMode::Normal;
                    Action::Render
                }
                _ => Action::Render,
            }
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            Action::FzfSelected(selected_string) => {
                self.select_project(selected_string)?;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let header = Row::new(vec!["Name", "ID"])
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .height(1)
            .bottom_margin(1);

        let footer = if let Some(selected) = self.table_state.selected() {
             if let Some(selected_project) = self.get_build_types().get(selected) {
                 Some(
                     Row::new(vec![format!(
                         "Root project: {}",
                         selected_project
                             .project_name
                             .as_deref()
                             .unwrap_or("N/A")
                     )])
                         .style(
                             Style::default()
                                 .fg(Color::Yellow)
                                 .add_modifier(Modifier::BOLD),
                         )
                         .height(1)
                         .top_margin(1),
                 )
             } else {
                 None
             }
        } else {
            None
        };

        let rows: Vec<Row> = self.get_build_types()
            .into_iter()
            .map(|build_type| {
                Row::new(vec![build_type.name.clone(), build_type.id.clone()])

            })
            .collect();

        let table = Table::new(rows, &[Constraint::Min(70), Constraint::Max(30)])
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Build Configurations"),
            )
            .column_spacing(1)
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");

        let table = if let Some(footer) = footer {
            table.footer(footer)
        } else {
            table
        };

        frame.render_stateful_widget(table, area, &mut self.table_state);

        if self.input_mode == InputMode::Editing {
            self.render_input_popup(frame, area);
        }
        Ok(())
    }
}
