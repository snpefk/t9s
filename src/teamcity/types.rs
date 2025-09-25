use serde::{Deserialize, Serialize};

// Look here for docs
// https://www.jetbrains.com/help/teamcity/rest/buildtype.html
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BuildType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "projectName")]
    pub project_name: Option<String>,
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    pub href: Option<String>,
    #[serde(rename = "webUrl")]
    pub web_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BuildTypes {
    pub count: u32,
    pub href: Option<String>,
    #[serde(rename = "buildType")]
    pub build_type: Vec<BuildType>,
}