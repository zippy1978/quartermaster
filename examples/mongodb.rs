#![cfg_attr(not(feature = "mongodb"), allow(unused_imports, dead_code))]

use std::time::Duration;

use async_trait::async_trait;
#[cfg(feature = "mongodb")]
use quartermaster::store::mongodb::MongoDBTaskStore;
use quartermaster::{manager::TaskManager, task::Task};
use std::sync::Arc;
use tokio::time::sleep;

#[cfg(feature = "mongodb")]
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

#[cfg(feature = "mongodb")]
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

#[cfg(not(feature = "mongodb"))]
fn main() {
    println!(
        r#"Please enable feature "mongodb", try:
    cargo run --features="mongodb" --example mongodb"#
    );
}
