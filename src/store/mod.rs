use std::fmt::Display;

use async_trait::async_trait;

use crate::task::Task;

pub mod memory;
#[cfg(test)]
pub mod memory_tests;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TaskStatus {
    Pending,
    Running,
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct TaskState {
    pub id: String,
    pub name: String,
    pub manager: String,
    pub instance: Option<String>,
    pub status: TaskStatus,
    pub creation_time: u64,
}

#[derive(Debug)]
pub enum TaskStoreError {
    Data(String),
    Io(String),
    NotFound(String),
}

impl Display for TaskStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// TaskStore.
/// In charge of keeping track of a manager task states.
#[async_trait]
pub trait TaskStore: Sized + Send + Sync + Clone {
    /// Create a new task store
    fn new(manager: &str, instance: Option<String>) -> Self;
    /// Create a new task state from a task.
    /// If successful, return a task state with a unique identifier.
    async fn save_state(&self, task: &dyn Task) -> Result<TaskState, TaskStoreError>;
    /// Delete task state.
    async fn delete_state(&self, task: &dyn Task) -> Result<(), TaskStoreError>;
    /// Retrieve a task state.
    async fn get_state(&self, task: &dyn Task) -> Result<Option<TaskState>, TaskStoreError>;
    /// Count running tasks.
    async fn count_tasks(&self) -> Result<usize, TaskStoreError>;
    /// Update task status.
    async fn update_status(&self, task: &dyn Task, status: TaskStatus) -> Result<(), TaskStoreError>;
    /// Clear store.
    async fn clear(&self) -> Result<(), TaskStoreError>;
    /// Return all the task states of the store.
    async fn get_all_states(&self) -> Result<Vec<TaskState>, TaskStoreError>;
}
