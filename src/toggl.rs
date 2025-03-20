use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Row {
    #[serde(rename = "User")]
    pub user: String,
    #[serde(rename = "Email")]
    pub email: String,
    #[serde(rename = "Client")]
    pub client: Option<String>,
    #[serde(rename = "Project")]
    pub project: Option<String>,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Start date")]
    pub start_date: String,
    #[serde(rename = "Start time")]
    pub start_time: String,
    #[serde(rename = "Duration")]
    pub duration: String,
}
