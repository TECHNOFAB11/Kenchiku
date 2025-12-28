use std::collections::HashMap;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

#[derive(Debug)]
pub struct Session {
    pub values: HashMap<String, serde_json::Value>,
    pub missing_values: Vec<String>,
    pub value_sender: std::sync::mpsc::Sender<HashMap<String, serde_json::Value>>,
    pub status_receiver: Option<Receiver<Status>>,
    pub join_handle: Option<JoinHandle<eyre::Result<String>>>,
}

#[derive(Debug)]
pub enum Status {
    MissingValue(MissingValueError),
}

#[derive(Debug, Clone)]
pub struct MissingValueError {
    pub name: String,
    pub r#type: String,
    pub description: String,
    pub choices: Option<Vec<String>>,
    pub error: Option<String>,
}

impl std::fmt::Display for MissingValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(err) = &self.error {
            write!(f, "Missing value: {} (Error: {})", self.name, err)
        } else {
            write!(f, "Missing value: {}", self.name)
        }
    }
}

impl std::error::Error for MissingValueError {}
