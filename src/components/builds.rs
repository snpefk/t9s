use super::Component;
use crate::teamcity::types::Build;
use crate::time::{
    format_datetime_to_human_readable_string, format_duration, parse_tc_datetime_to_epoch,
};
use crate::{action::Action, config::Config};
use color_eyre::eyre::anyhow;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table, TableState};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct Builds {
    title: String,
    items: Vec<Build>,
    table_state: TableState,
    last_events: Vec<KeyEvent>,
    pub filter_string: Option<String>,
    pub action_tx: Option<UnboundedSender<Action>>,
}

impl Builds {
    pub fn new(project_title: String, builds: Vec<Build>) -> Self {
        Self {
            title: project_title,
            items: builds,
            filter_string: None,
            ..Self::default()
        }
    }

    fn get_items(&self) -> Vec<Build> {
        self.items.clone()
    }

    fn move_down(&mut self) {
        let items = self.get_items();
        if items.is_empty() {
            self.table_state.select(None);
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= items.len() - 1 {
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
        let items = self.get_items();
        if items.is_empty() {
            self.table_state.select(None);
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn move_begin(&mut self) {
        if self.get_items().is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(0));
        }
    }

    fn move_end(&mut self) {
        let items = self.get_items();
        if items.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(items.len() - 1));
        }
    }

    fn open_selected_url(&mut self) {
        if let Some(i) = self.table_state.selected() {
            if let Some(bt) = self.get_items().get(i) {
                if let Some(url) = &bt.web_url {
                    let _ = open::that(url);
                }
            }
        }
    }

    fn select_build(&mut self, selected_string: String) {
        if let Some((i, _)) = self.get_items().iter().enumerate().find(|(_, b)| {
            let label = format!(
                "#{} {}",
                b.id.map(|x| x.to_string()).unwrap_or_default(),
                b.build_number.clone().unwrap_or_default(),
            );
            label == selected_string
        }) {
            self.table_state.select(Some(i));
        }
    }
}

impl Component for Builds {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, _config: Config) -> color_eyre::Result<()> {
        Ok(())
    }

    fn init(&mut self, _area: Size) -> color_eyre::Result<()> {
        if !self.items.is_empty() {
            self.table_state.select(Some(0));
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        self.last_events.push(key);

        let action = match key.code {
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
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
                Action::Render
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
                Action::Render
            }
            KeyCode::Char('f') => {
                let items: Vec<String> = self
                    .get_items()
                    .iter()
                    .map(|build| {
                        format!(
                            "#{} {} [{}]",
                            build.id.unwrap_or_default(),
                            build.build_number.clone().unwrap_or_default(),
                            build.build_type_id.clone().unwrap_or_default()
                        )
                    })
                    .collect();
                Action::Fzf(items)
            }
            KeyCode::Char('o') => {
                self.open_selected_url();
                Action::Render
            }
            KeyCode::Esc | KeyCode::Char('h') => Action::ShowProjects,
            _ => Action::Render,
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::FzfSelected(selected_string) => {
                self.select_build(selected_string);
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let header = Row::new(vec![
            "Number",
            "Branch",
            "Status",
            "Last Changes",
            "Start time",
            "Duration",
        ])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .height(1)
        .bottom_margin(1);

        let rows: Vec<Row> = self
            .get_items()
            .into_iter()
            .map(|build| {
                let number = build.build_number.unwrap_or_default();
                let branch = build.branch_name.unwrap_or_default();
                let status_text = build
                    .status_text
                    .clone()
                    .or(build.status.clone())
                    .unwrap_or_default();

                let changes = build
                    .changes
                    .as_ref()
                    .and_then(|c| c.change.clone())
                    .unwrap_or_default();

                let last_changes = if !changes.is_empty() {
                    let users: Vec<&String> =
                        changes.iter().filter_map(|c| c.username.as_ref()).collect();
                    if users.len() == 0 {
                        format!("⚠️ {} Changes from 0 users", changes.len())
                    } else if users.len() == 1 {
                        format!("{}: {}", users[0], changes.len())
                    } else {
                        format!("{} Changes", changes.len())
                    }
                } else {
                    "No changes".to_string()
                };

                let start_datetime = build
                    .start_date
                    .as_ref()
                    .and_then(|s| format_datetime_to_human_readable_string(s).ok())
                    .unwrap_or_default();

                let duration = {
                    if let Some(ref start) = build.start_date {
                        let start_epoch = parse_tc_datetime_to_epoch(start);
                        let end_epoch = if let Some(ref finish) = build.finish_date {
                            parse_tc_datetime_to_epoch(finish)
                        } else {
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs() as i64)
                                .map_err(|e| anyhow!(e))
                        };

                        match (start_epoch, end_epoch) {
                            (Ok(s), Ok(e)) if e >= s => format_duration(e - s),
                            _ => Ok(String::new()),
                        }
                    } else {
                        Ok(String::new())
                    }
                };

                let mut row = Row::new(vec![
                    number,
                    branch,
                    status_text.clone(),
                    last_changes,
                    start_datetime,
                    duration.unwrap_or_default(),
                ]);

                // if build status is None then it's in queue state
                let is_failed = if let Some(status) = build.status {
                    match status.as_str() {
                        "FAILURE" | "UNKNOWN" => { true }
                        _ => false,
                    }
                } else {
                    false
                };

                if is_failed {
                    row = row.style(Style::default().fg(Color::Red));
                }
                row
            })
            .collect();

        let table = Table::new(
            rows,
            &[
                Constraint::Max(20),    // Number
                Constraint::Length(30), // Branch
                Constraint::Min(20),    // Status text
                Constraint::Max(40),    // Last Changes
                Constraint::Length(13), // Start time (HH:MM)
                Constraint::Length(9),  // Duration (M:SS or H:MM:SS)
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Builds — {}", self.title)),
        )
        .column_spacing(1)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);

        Ok(())
    }
}
