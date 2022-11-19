use std::fmt::Display;

#[cfg(feature = "mongodb")]
use mongodb::bson::oid::ObjectId;
#[cfg(feature = "mongodb")]
use serde::{Deserialize, Serialize};

/// Represent a task state status.
#[cfg_attr(
    feature = "serde",
    derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)
)]
#[cfg_attr(not(feature = "serde"), derive(Debug, Clone, Eq, PartialEq, Hash))]
pub enum TaskStatus {
    Pending,
    Running,
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represent a task state.
#[cfg_attr(
    feature = "serde",
    derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)
)]
#[cfg_attr(not(feature = "serde"), derive(Debug, Clone, Eq, PartialEq, Hash))]
pub struct TaskState {
    #[cfg(not(feature = "mongodb"))]
    pub id: Option<String>,
    #[cfg(feature = "mongodb")]
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub task_id: String,
    pub task_name: String,
    pub task_manager: String,
    pub instance: Option<String>,
    pub status: TaskStatus,
    pub creation_time: u64,
}
