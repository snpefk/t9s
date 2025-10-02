use serde::{Deserialize, Serialize};
use strum::Display;
use std::path::PathBuf;

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
    Fzf { options: Vec<String> },
    FzfSelected { selected: String },
    Pager { file: PathBuf },
    // Builds
    LoadBuilds { project_id: String, title: String },
    ShowBuilds { title: String, items: Vec<Build> },
    LoadBuildLog { build_id: i64 },
    // Projects
    ShowProjects,
}
