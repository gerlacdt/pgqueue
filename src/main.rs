use pgqueue::sql::{listen, Message, MessageEntity, Messenger, Payload};
use serde_json::json;
use sqlx::PgPool;
use std::sync::mpsc::{self, Receiver, Sender};

#[tokio::main]
async fn main() {
    let _ = foo().await.unwrap();
}

// more information
// https://github.com/launchbadge/sqlx/blob/main/examples/postgres/listen/src/main.rs
// https://www.fforward.ai/blog/posts/postgres-task-queues-the-secret-weapon-killing-specialized-queue-services
async fn foo() -> Result<(), sqlx::Error> {
    let pool = get_pool().await;
    let pool2 = pool.clone();

    let (tx, rx): (Sender<MessageEntity>, Receiver<MessageEntity>) = mpsc::channel();

    let process_fn = move |entity: &MessageEntity| {
        if let Err(_) = tx.send(entity.clone()) {
            println!("ERROR sending from process_fn()");
        }
    };

    let (start_tx, start_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    let start_tx2 = start_tx.clone();
    let _ = tokio::spawn(async move {
        start_tx.send(()).unwrap();
        listen(pool, process_fn)
            .await
            .expect("ERROR listening to pq task queue");
    });

    let rx_handle = tokio::spawn(async move {
        start_tx2.send(()).unwrap();
        for _ in 0..3 {
            match rx.recv() {
                Ok(msg) => println!("recv(): {:?}", msg),
                Err(err) => println!("Err: {:?}", err),
            }
        }
    });

    // NOTIFY channel, with add() new message into channel
    start_rx.recv().unwrap();
    start_rx.recv().unwrap();
    let messenger = Messenger::new(pool2);
    for i in 0..3 {
        let payload = Payload {
            version: 1,
            kind: "Command".to_owned(),
            message: format!("my new message notification {}", i),
        };
        let msg = Message {
            payload: json!(payload),
        };
        messenger.add(msg).await.unwrap();
    }

    rx_handle.await.unwrap();

    Ok(())
}

async fn get_pool() -> PgPool {
    let conn_str = std::env::var("DATABASE_URL").expect("Env Var DATABASE_URL must be set");
    PgPool::connect(&conn_str).await.unwrap()
}

// write a process_fn with channels and wait for results
// we should get rid of sleep()
