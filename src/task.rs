use async_trait::async_trait;

/// Task.
/// Defines a task to run.
#[async_trait]
pub trait Task<O = ()>: Send + Sync where O: Default{
    /// Return the name of the task.
    fn name(&self) -> String;
    /// Return the unique id of the task.
    /// Two tasks with the same name and the same id are considered as equal.
    fn id(&self) -> String;
    /// Task execution.
    async fn run(&self) -> O {
        O::default()
    }
}
