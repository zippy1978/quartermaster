use async_trait::async_trait;

use crate::{store::TaskStatus, task::Task};

use super::{memory::InMemoryTaskStore, TaskStore};

struct TestTask {
    pub id: String,
}

#[async_trait]
impl Task for TestTask {
    fn name(&self) -> String {
        "test_task".to_string()
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    async fn run(&self) {
        // Nothing
    }
}

#[tokio::test]
async fn create_state() {
    let mem_store = InMemoryTaskStore::new("test_manager", None);
    let state1 = mem_store
        .save_state(&TestTask {
            id: "1".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(state1.id, "1");
    let state2 = mem_store
        .save_state(&TestTask {
            id: "2".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(state2.id, "2");
    assert_eq!(mem_store.count_tasks().await.unwrap(), 2 as usize);
}

#[tokio::test]
async fn get_state_found() {
    let mem_store = InMemoryTaskStore::new("test_manager", None);
    let task = TestTask {
        id: "1".to_string(),
    };
    mem_store.save_state(&task).await.unwrap();
    assert!(mem_store.get_state(&task).await.unwrap().is_some());
}

#[tokio::test]
async fn get_state_not_found() {
    let mem_store = InMemoryTaskStore::new("test_manager", None);
    let task = TestTask {
        id: "1".to_string(),
    };
    mem_store.save_state(&task).await.unwrap();
    assert!(mem_store
        .get_state(&TestTask {
            id: "2".to_string(),
        })
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn delete_state() {
    let mem_store = InMemoryTaskStore::new("test_manager", None);
    let task = TestTask {
        id: "1".to_string(),
    };
    mem_store.save_state(&task).await.unwrap();
    assert_eq!(mem_store.count_tasks().await.unwrap(), 1 as usize);
    mem_store.delete_state(&task).await.unwrap();
    assert_eq!(mem_store.count_tasks().await.unwrap(), 0 as usize);
}

#[tokio::test]
async fn update_state() {
    let mem_store = InMemoryTaskStore::new("test_manager", None);
    let task = TestTask {
        id: "1".to_string(),
    };
    mem_store.save_state(&task).await.unwrap();
    mem_store
        .update_status(&task, TaskStatus::Running)
        .await
        .unwrap();
    let state = mem_store.get_state(&task).await.unwrap().unwrap();
    assert_eq!(state.status, TaskStatus::Running);
}

#[tokio::test]
async fn clear() {
    let mem_store = InMemoryTaskStore::new("test_manager", None);
    mem_store
        .save_state(&TestTask {
            id: "1".to_string(),
        })
        .await
        .unwrap();
    mem_store
        .save_state(&TestTask {
            id: "2".to_string(),
        })
        .await
        .unwrap();
    mem_store.clear().await.unwrap();
    assert_eq!(mem_store.count_tasks().await.unwrap(), 0 as usize);
}
