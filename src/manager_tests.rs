use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::{sync::RwLock, time::sleep};

use crate::{manager::TaskManager, store::memory::InMemoryTaskStore, task::Task};

struct TestTask {
    pub id: String,
    pub sleep_millis: u64,
    pub results: Arc<RwLock<Vec<String>>>,
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
        sleep(Duration::from_millis(self.sleep_millis)).await;
        self.results.write().await.push(self.id.clone());
    }
}

#[tokio::test]
async fn run_serial() {
    let results = Arc::new(RwLock::new(vec![]));

    let manager = TaskManager::new(InMemoryTaskStore::new("manager"), 1);

    manager
        .run(Box::new(TestTask {
            id: "1".to_string(),
            sleep_millis: 10,
            results: results.clone(),
        }))
        .await;
    manager
        .run(Box::new(TestTask {
            id: "2".to_string(),
            sleep_millis: 50,
            results: results.clone(),
        }))
        .await;
    manager
        .run(Box::new(TestTask {
            id: "3".to_string(),
            sleep_millis: 5,
            results: results.clone(),
        }))
        .await;

    manager.stop().await;

    manager.start_blocking().await;

    assert_eq!(results.read().await.len(), 3);
    assert_eq!(results.read().await[0], "1");
    assert_eq!(results.read().await[1], "2");
    assert_eq!(results.read().await[2], "3");
}


#[tokio::test]
async fn run_parallel() {
    let results = Arc::new(RwLock::new(vec![]));

    let manager = TaskManager::new(InMemoryTaskStore::new("manager"), 2);

    manager
        .run(Box::new(TestTask {
            id: "1".to_string(),
            sleep_millis: 10,
            results: results.clone(),
        }))
        .await;
    manager
        .run(Box::new(TestTask {
            id: "2".to_string(),
            sleep_millis: 50,
            results: results.clone(),
        }))
        .await;
    manager
        .run(Box::new(TestTask {
            id: "3".to_string(),
            sleep_millis: 5,
            results: results.clone(),
        }))
        .await;

    manager.stop().await;

    manager.start_blocking().await;

    assert_eq!(results.read().await.len(), 3);
    assert_eq!(results.read().await[0], "1");
    assert_eq!(results.read().await[1], "3");
    assert_eq!(results.read().await[2], "2");
}


#[tokio::test]
async fn get_state() {
    let results = Arc::new(RwLock::new(vec![]));

    let manager = TaskManager::new(InMemoryTaskStore::new("manager"), 2);

    manager
        .run(Box::new(TestTask {
            id: "1".to_string(),
            sleep_millis: 10,
            results: results.clone(),
        }))
        .await;
    manager
        .run(Box::new(TestTask {
            id: "2".to_string(),
            sleep_millis: 50,
            results: results.clone(),
        }))
        .await;
    manager
        .run(Box::new(TestTask {
            id: "3".to_string(),
            sleep_millis: 5,
            results: results.clone(),
        }))
        .await;

    let state = manager.get_state().await;

    assert_eq!(state.len(), 3);
    
}