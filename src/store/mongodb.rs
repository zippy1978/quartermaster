use async_trait::async_trait;
use std::sync::Arc;
use futures::TryStreamExt;

use crate::util::now_secs;

use super::{
    state::{TaskState, TaskStatus},
    TaskStore, TaskStoreError,
};

use mongodb::{
    bson::{doc, Bson},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

impl From<mongodb::error::Error> for TaskStoreError {
    fn from(err: mongodb::error::Error) -> Self {
        Self::Data(err.to_string())
    }
}

impl From<TaskStatus> for Bson {
    fn from(status: TaskStatus) -> Self {
        Bson::from(status.to_string())
    }
}

/// In Memory (thread safe) task store implementation.
#[derive(Clone)]
pub struct MongoDBTaskStore {
    manager: String,
    instance: String,
    db: Arc<Database>,
}

impl MongoDBTaskStore {
    pub fn new(manager_name: &str, instance_name: &str, db: Arc<Database>) -> Self {
        Self {
            manager: manager_name.to_string(),
            instance: instance_name.to_string(),
            db,
        }
    }

    fn collection(&self) -> Collection<TaskState> {
        self.db.collection("TaskState")
    }
}

#[async_trait]
impl TaskStore for MongoDBTaskStore {
    fn manager_name(&self) -> String {
        self.manager.to_string()
    }

    async fn init(&self) -> Result<(), TaskStoreError> {
        let col = self.collection();
        // Index: task_id + task_name
        let model = IndexModel::builder()
            .keys(doc! {"task_id": 1u32, "task_name": 1u32})
            .options(IndexOptions::builder().unique(true).build())
            .build();
        col.create_index(model).await?;
        Ok(())
    }

    async fn save_state(
        &self,
        task: &dyn crate::task::Task,
    ) -> Result<super::TaskState, super::TaskStoreError> {
        // Create state
        let state = TaskState {
            id: None,
            task_id: task.id(),
            task_name: task.name(),
            task_manager: self.manager.to_string(),
            instance: Some(self.instance.to_string()),
            status: super::TaskStatus::Pending,
            creation_time: now_secs(),
        };

        // Store state
        let col = self.collection();
        let inserted = col.insert_one(state).await?;
        let filter = doc! {"_id": inserted.inserted_id};
        let result = col.find_one(filter).await?;

        Ok(result.unwrap())
    }

    async fn delete_state(
        &self,
        task: &dyn crate::task::Task,
    ) -> Result<(), super::TaskStoreError> {
        // Retrieve task state
        if let Some(state) = self.get_state(task).await? {
            // Delete
            let col = self.collection();
            let filter = doc! {"_id": state.id};
            col.find_one_and_delete(filter).await?;
        }
        Ok(())
    }

    async fn get_state(
        &self,
        task: &dyn crate::task::Task,
    ) -> Result<Option<super::TaskState>, super::TaskStoreError> {
        let col = self.collection();
        let state = col
            .find_one(
                doc! {"task_manager": &self.manager, "task_name": task.name(), "task_id": task.id()}
            )
            .await?;
        Ok(state)
    }

    async fn count_tasks(&self) -> Result<usize, super::TaskStoreError> {
        // find for current manager
        let col = self.collection();
        let count = col.count_documents(doc! {"instance": &self.instance}).await?;
        Ok(count as usize)
    }

    async fn update_status(
        &self,
        task: &dyn crate::task::Task,
        status: super::TaskStatus,
    ) -> Result<(), super::TaskStoreError> {
        // Retrieve task state
        if let Some(state) = self.get_state(task).await? {
            // Update if found
            let col = self.collection();
            let filter = doc! {"_id": state.id};
            let update = doc! {"$set": {
                "status": status}
            };
            col.update_one(filter, update).await?;
        }
        Ok(())
    }

    async fn clear(&self) -> Result<(), super::TaskStoreError> {
        let col = self.collection();
        let filter = doc! {"instance": &self.instance};
        col.delete_many(filter).await?;
        Ok(())
    }

    async fn get_all_states(&self) -> Result<Vec<super::TaskState>, super::TaskStoreError> {
        let col = self.collection();
        let filter = doc! {"instance": &self.instance};
        let states = col.find(filter).await?.try_collect().await?;
        Ok(states)
    }
}
