/*!
Quartermaster, a Rust task manager.

# Basic usage

This example uses an in memory store to track task states.

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
    let tm = TaskManager::new(InMemoryTaskStore::new("manager"), 2);

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
        .for_each(|s| println!("name = [{}], id = [{}], creation time = [{}], status = [{}]", s.task_name, s.task_id, s.creation_time, s.status));
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

# MongoDB

The library allows persisting states into a MongoDB collection.
This is appropriate to share states across muntiple server instances, and then make sure a same task with the same id cannot be run at the the same time in the cluster.

Here is a modified version of the basic example using MongoDB.

Note that `mongodb` feature must explicitly be enabled on the crate to make it work.

```rust
use std::time::Duration;

use async_trait::async_trait;
use quartermaster::store::mongodb::MongoDBTaskStore;
use quartermaster::{manager::TaskManager, task::Task};
use std::sync::Arc;
use tokio::time::sleep;

use mongodb::Client;

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
    // Create database connection
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    let db = Arc::new(client.database("quartermaster"));

    // Create task manager
    // Instance name should be unique to your server instance
    let tm = TaskManager::new(MongoDBTaskStore::new("manager", "instance", db.clone()), 2);

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

#![allow(
    clippy::needless_pass_by_value,
    clippy::new_without_default,
    clippy::new_ret_no_self
)]

 

pub mod task;
pub mod manager;
pub mod store;
mod util;

#[cfg(test)]
pub mod manager_tests;