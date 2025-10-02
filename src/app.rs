use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::components::builds::Builds;
use crate::components::projects::Projects;
use crate::teamcity::TeamCityClient;
use crate::teamcity::types::{Build, BuildType};
use crate::{
    action::Action,
    components::{Component, fps::FpsCounter, home::Home},
    config::Config,
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    client: TeamCityClient,
    build_types: Vec<BuildType>,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}

impl App {
    pub fn new(client: TeamCityClient, build_types: Vec<BuildType>) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        Ok(Self {
            components: vec![Box::new(Projects::new(build_types.clone()))],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode: Mode::Home,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            client,
            build_types: build_types.clone(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(4.0)
            .frame_rate(1.0);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        loop {
            let action_tx = self.action_tx.clone();
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };
        match keymap.get(&vec![key]) {
            Some(action) => {
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::Fzf { ref options } => {
                    let selected: String = tui.run_fzf(&options)?;
                    self.action_tx.send(Action::FzfSelected { selected })?;
                    ()
                }
                Action::LoadBuilds {
                    ref project_id,
                    ref title,
                } => {
                    self.components = vec![Box::new(Builds::new(title.clone(), vec![]))];

                    for component in self.components.iter_mut() {
                        component.register_action_handler(self.action_tx.clone())?;
                        component.register_config_handler(self.config.clone())?;
                        component.init(tui.size()?)?;
                    }
                    self.render(tui)?;

                    let client = self.client.clone();
                    let tx = self.action_tx.clone();
                    let title = title.clone(); // Clone title here to create an owned value for the closure
                    let project_id = project_id.clone();

                    tokio::spawn(async move {
                        match client.get_builds_by_project(&project_id).await {
                            Ok(items) => {
                                let _ = tx.send(Action::ShowBuilds {
                                    title: title.clone(),
                                    items,
                                });
                            }
                            Err(e) => {
                                let error_msg = format!(
                                    "Failed to fetch builds for project {}: {}",
                                    project_id, e
                                );
                                let _ = tx.send(Action::Error(error_msg));
                            }
                        }
                    });
                }
                Action::ShowBuilds {
                    ref title,
                    ref items,
                } => {
                    self.components = vec![Box::new(Builds::new(title.clone(), items.clone()))];

                    for component in self.components.iter_mut() {
                        component.register_action_handler(self.action_tx.clone())?;
                        component.register_config_handler(self.config.clone())?;
                        component.init(tui.size()?)?;
                    }
                    self.render(tui)?;
                }
                Action::ShowProjects => {
                    self.components = vec![Box::new(Projects::new(self.build_types.clone()))];
                    for component in self.components.iter_mut() {
                        component.register_action_handler(self.action_tx.clone())?;
                        component.register_config_handler(self.config.clone())?;
                        component.init(tui.size()?)?;
                    }
                    self.render(tui)?;
                }
                _ => {}
            }

            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }
}
