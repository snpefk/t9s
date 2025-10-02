use serde::{Deserialize, Serialize};

// Look here for docs
// https://www.jetbrains.com/help/teamcity/rest/buildtype.html
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
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
    #[serde(rename = "type")]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BuildTypes {
    pub count: u32,
    pub href: Option<String>,
    #[serde(rename = "buildType")]
    pub build_type: Vec<BuildType>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Change {
    pub username: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Changes {
    #[serde(rename = "change")]
    pub change: Option<Vec<Change>>,
}

// Build entity docs:
// https://www.jetbrains.com/help/teamcity/rest/build.html
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Build {
    pub id: Option<i64>,
    #[serde(rename = "buildTypeId")]
    pub build_type_id: Option<String>,
    #[serde(rename = "number")]
    pub build_number: Option<String>,
    pub status: Option<String>, // SUCCESS, FAILURE, etc.
    pub state: Option<String>,  // queued, running, finished
    pub href: Option<String>,
    #[serde(rename = "webUrl")]
    pub web_url: Option<String>,
    #[serde(rename = "branchName")]
    pub branch_name: Option<String>,
    #[serde(rename = "statusText")]
    pub status_text: Option<String>,
    #[serde(rename = "queuedDate")]
    pub queued_date: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "finishDate")]
    pub finish_date: Option<String>,
    pub changes: Option<Changes>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Builds {
    pub count: Option<u32>,
    pub href: Option<String>,
    #[serde(rename = "build")]
    pub build: Vec<Build>,
}