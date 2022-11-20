use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    store::{
        state::{TaskState, TaskStatus},
        TaskStore,
    },
    task::Task,
};

type TaskQueue = deadqueue::unlimited::Queue<Box<dyn Task>>;

/// Stop task is a system task.
/// It is used to shutdown the task manger.
struct StopTask {}

#[async_trait]
impl Task for StopTask {
    fn name(&self) -> String {
        "stop".to_string()
    }

    fn id(&self) -> String {
        "stop".to_string()
    }

    async fn run(&self) {}
}

/// Task manager.
/// In charge of handling tasks by assigning them to worker threads.
pub struct TaskManager<S>
where
    S: TaskStore,
{
    /// Task queue.
    queue: Arc<TaskQueue>,
    /// Task manager name.
    name: String,
    /// Number of workers for this task manager.
    worker_count: usize,
    /// Task store to track states
    store: Arc<S>,
    /// Task manager state
    started: Arc<RwLock<bool>>,
}

impl<S: TaskStore + 'static> TaskManager<S> {
    /// Create a new task manager.
    pub fn new(store: S, worker_count: usize) -> Self {
        Self {
            queue: Arc::new(TaskQueue::new()),
            name: store.manager_name(),
            worker_count,
            store: Arc::new(store),
            started: Arc::new(RwLock::new(false)),
        }
    }

    /// Run an task.
    pub async fn run(&self, task: Box<dyn Task + Send + Sync>) {
        // Check if task is already known
        match self.store.get_state(task.as_ref()).await {
            Ok(r) => {
                if r.is_some() {
                    log::debug!(
                        "task `{}` with id `{}` already exists",
                        task.name(),
                        task.id()
                    );
                    return;
                }
            }
            Err(err) => {
                log::error!(
                    "failed to retrieve task `{}` with id `{}` state: {}",
                    task.name(),
                    task.id(),
                    err.to_string()
                );
                return;
            }
        };

        // Add task state to store
        if let Some(err) = self.store.save_state(task.as_ref()).await.err() {
            log::error!(
                "failed to save task `{}` with id `{}` state: {}",
                task.name(),
                task.id(),
                err.to_string()
            );
        }

        // Add task to queue
        self.queue.push(task);
    }

    /// Start task manager.
    pub async fn start(&self) {
        self.start_with_options(false).await;
    }

    /// Start task manager.
    /// Function will block until all worker threads are terminated.
    pub async fn start_blocking(&self) {
        self.start_with_options(true).await;
    }

    /// Start task manager with options.
    /// If started with join set to true,
    /// function will block until all worker threads are terminated.
    async fn start_with_options(&self, join: bool) {
        // Check if already started
        if *self.started.read().await {
            log::warn!("task manager `{}` is already stared", self.name);
            return;
        }

        log::info!(
            "starting task manager `{}`, with {} worker(s)",
            self.name,
            self.worker_count
        );

        // initialized store
        if let Some(err) = self.store.init().await.err() {
            log::error!(
                "task manager `{}` failed to initialize store: {}",
                self.name,
                err.to_string()
            );
        }

        // Clear state
        self.clear().await;

        let mut handles = vec![];

        // Start workers
        for worker in 0..self.worker_count {
            let queue = self.queue.clone();
            let store = self.store.clone();
            let name = self.name.clone();
            let started = self.started.clone();
            *started.write().await = true;
            let handle = tokio::spawn(async move {
                while *started.read().await {
                    let task = queue.pop().await;

                    if task.name() == "stop" {
                        *started.write().await = false;
                    } else {
                        // Update task state to 'running'
                        if let Some(err) = store
                            .update_status(task.as_ref(), TaskStatus::Running)
                            .await
                            .err()
                        {
                            log::error!(
                                "failed to update task `{}` with id `{}` state: {}",
                                task.name(),
                                task.id(),
                                err.to_string()
                            );
                        }

                        log::info!(
                            "starting task `{}` with id `{}` on task manager `{}`, worker: {}",
                            task.name(),
                            task.id(),
                            name,
                            worker
                        );

                        // Run task
                        task.run().await;

                        log::info!(
                            "finished task `{}` with id `{}` on task manager `{}`, worker: {}",
                            task.name(),
                            task.id(),
                            name,
                            worker
                        );

                        // Clear task state
                        if let Some(err) = store.delete_state(task.as_ref()).await.err() {
                            log::error!(
                                "failed to clear task `{}` with id `{}` state: {}",
                                task.name(),
                                task.id(),
                                err.to_string()
                            );
                        }
                    }
                }
            });
            handles.push(handle);
        }

        // Join threads to block until workers are terminated
        if join {
            let mut results = Vec::with_capacity(handles.len());
            for handle in handles {
                results.push(handle.await.unwrap());
            }
        }
    }

    /// Stop task manager.
    pub async fn stop(&self) {
        self.queue.push(Box::new(StopTask {}));
    }

    /// Clear task manager task states.
    pub async fn clear(&self) {
        if let Some(err) = self.store.clear().await.err() {
            log::error!(
                "failed to clear task manager `{}` state : {}",
                self.name,
                err.to_string()
            );
        }
    }

    /// Get task manager state
    pub async fn get_state(&self) -> Vec<TaskState> {
        match self.store.get_all_states().await {
            Ok(states) => states,
            Err(err) => {
                log::error!(
                    "failed to retrieve task manager `{}` state : {}",
                    self.name,
                    err.to_string()
                );
                vec![]
            }
        }
    }
}
