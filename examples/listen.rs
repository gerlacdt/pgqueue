use pgqueue::sql::{Message, MessageEntity, Messenger, Payload};
use serde_json::json;
use sqlx::{postgres::PgListener, PgPool};
use std::sync::mpsc::{self, Receiver, Sender};

static QUEUE_NAME: &'static str = "queue_notifications";

#[tokio::main]
async fn main() {
    let _ = example().await.unwrap();
}

async fn example() -> Result<(), sqlx::Error> {
    let pool = get_pool().await;
    // clean DB, so only new message get processed
    sqlx::query!("DELETE FROM messages").execute(&pool).await?;
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
            let msg = rx.recv().unwrap();
            println!("recv(): {:?}", msg);
        }
    });

    // NOTIFY channel, with add() new message into channel
    start_rx.recv().unwrap();
    start_rx.recv().unwrap();
    let messenger = Messenger::new(pool2);
    for i in 0..3 {
        let payload = Payload {
            version: 1,
            kind: "COMMAND".to_owned(),
            message: format!("message: {}", i),
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

async fn listen<F>(pool: PgPool, process_fn: F) -> Result<(), sqlx::Error>
where
    F: Fn(&MessageEntity),
{
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen(QUEUE_NAME).await?;
    let messenger = Messenger::new(pool.clone());
    loop {
        let _notification = listener.recv().await;
        messenger.process_next(&process_fn).await?;
    }
}
