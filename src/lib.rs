/*!
Quartermaster, a Rust task manager.

# Basic usage
```rust
use std::time::Duration;

use async_trait::async_trait;
use quartermaster::{manager::TaskManager, store::memory::InMemoryTaskStore, task::Task};
use tokio::time::sleep;

// A simple task printing hello after a delay
// name + id make a task unique
// A task manager will refuse to run the same task while it is already running or pending
struct DelayedHelloTask {
    name: String,
    delay_millis: u64,
}

#[async_trait]
impl Task for DelayedHelloTask {
    // Define the task name
    fn name(&self) -> String {
        "delayed_hello".to_string()
    }

    // Define the task id (here, the person to greet,
    // but could be the id of the data the task is processing)
    fn id(&self) -> String {
        self.name.clone()
    }

    // Task code
    async fn run(&self) {
        sleep(Duration::from_millis(self.delay_millis)).await;
        println!("Hello {} !", self.name);
    }
}

#[tokio::main]
async fn main() {
    // Create task manager with in memory state storage and 3 workers
    let tm = TaskManager::<InMemoryTaskStore>::new("manager", 2);

    // Run tasks on the manager
    tm.run(Box::new(DelayedHelloTask {
        name: "Bart".to_string(),
        delay_millis: 5000,
    }))
    .await;
    tm.run(Box::new(DelayedHelloTask {
        name: "Homer".to_string(),
        delay_millis: 1000,
    }))
    .await;

    // Get task manager state
    tm.get_state()
        .await
        .iter()
        .for_each(|s| println!("name = [{}], id = [{}], creation time = [{}], status = [{}]", s.name, s.id, s.creation_time, s.status));
    // Output
    //name = [delayed_hello], id = [Homer], creation time = [1668816441], status = [Pending]
    //name = [delayed_hello], id = [Bart], creation time = [1668816441], status = [Pending]


    // Stop the task manager
    tm.stop().await;

    // Start the manager and block until stopped
    // (required in this case, otherwise program will exit before tasks are run)
    // Use tm.start().await to start without blocking.
    tm.start_blocking().await;

    // Result output:
    // Hello Homer !
    // Hello Bart !
}

```

 */

pub mod task;
pub mod manager;
pub mod store;
mod util;

#[cfg(test)]
pub mod manager_tests;