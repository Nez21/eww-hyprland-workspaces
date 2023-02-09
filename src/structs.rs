use serde::Deserialize;
use std::collections;

#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub id: u8,
    #[serde(rename = "monitor")]
    pub monitor_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub name: String,
    #[serde(rename = "activeWorkspace")]
    pub active_workspace: Workspace,
    pub focused: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub workspaces: collections::HashMap<u8, String>,
    pub template: String,
    #[serde(rename = "bodyTemplate")]
    pub body_template: String,
}
