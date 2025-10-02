use serde::{Deserialize, Serialize};
use strum::Display;

use crate::teamcity::types::Build;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    // Terminal-related actions
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    // General UI actions
    Error(String),
    Help,
    // External actions
    Fzf(Vec<String>),
    FzfSelected(String),
    // Builds
    LoadBuilds { project_id: String, title: String },
    ShowBuilds { title: String, items: Vec<Build> },
    // Projects
    ShowProjects,
}
