use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{task::Task, util::now_secs};

use super::{TaskState, TaskStatus, TaskStore, TaskStoreError};

/// In Memory (thread safe) task store implementation.
#[derive(Clone)]
pub struct InMemoryTaskStore {
    manager: String,
    states: Arc<RwLock<HashSet<TaskState>>>,
}

impl InMemoryTaskStore {
    /// Create a new task store
    pub fn new(manager_name: &str) -> Self {
        Self {
            manager: manager_name.to_string(),
            states: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

#[async_trait]
impl TaskStore for InMemoryTaskStore {
    fn manager_name(&self) -> String {
        self.manager.clone()
    }
    
    async fn init(&self) -> Result<(), TaskStoreError> {
        // Nothing to initialize
        Ok(())
    }
    async fn save_state<O: Default>(&self, task: &dyn Task<O>) -> Result<TaskState, TaskStoreError> {
        // Insert new task state
        let state = TaskState {
            id: None,
            task_id: task.id(),
            task_name: task.name(),
            task_manager: self.manager.to_string(),
            instance: None,
            status: super::TaskStatus::Pending,
            creation_time: now_secs(),
        };
        self.states.write().await.insert(state.clone());

        Ok(state)
    }

    async fn delete_state<O: Default>(&self, task: &dyn Task<O>) -> Result<(), TaskStoreError> {
        match self.get_state(task).await? {
            Some(s) => {
                self.states.write().await.remove(&s);
                Ok(())
            }
            None => Err(TaskStoreError::NotFound(format!(
                "task {} with id {} was not found",
                task.name(),
                task.id()
            ))),
        }
    }

    async fn get_state<O: Default>(&self, task: &dyn Task<O>) -> Result<Option<TaskState>, TaskStoreError> {
        Ok(self
            .states
            .read()
            .await
            .clone()
            .into_iter()
            .find(|s| s.task_id == task.id() && s.task_name == task.name()))
    }

    async fn count_tasks(&self) -> Result<usize, TaskStoreError> {
        Ok(self.states.read().await.len())
    }

    async fn update_status<O: Default>(
        &self,
        task: &dyn Task<O>,
        status: TaskStatus,
    ) -> Result<(), TaskStoreError> {
        match self.get_state(task).await? {
            Some(s) => {
                let mut new_state = s.clone();
                new_state.status = status;
                self.states.write().await.remove(&s);
                self.states.write().await.insert(new_state);
                Ok(())
            }
            None => Err(TaskStoreError::NotFound(format!(
                "task {} with id {} was not found",
                task.name(),
                task.id()
            ))),
        }
    }

    async fn clear(&self) -> Result<(), TaskStoreError> {
        self.states.write().await.clear();
        Ok(())
    }

    async fn get_all_states(&self) -> Result<Vec<TaskState>, TaskStoreError> {
        Ok(self.states.read().await.clone().into_iter().collect())
    }
}
