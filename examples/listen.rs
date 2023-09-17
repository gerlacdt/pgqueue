use pgqueue::sql::{Message, MessageEntity, Messenger};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgListener, PgPool};
use std::sync::mpsc::{self, Receiver, Sender};

static QUEUE_NAME: &'static str = "queue_notifications";

#[tokio::main]
async fn main() {
    let _ = example().await.unwrap();
}

#[derive(Deserialize, Serialize)]
pub struct CustomPayload {
    pub number: i64,
}

async fn example() -> Result<(), sqlx::Error> {
    let pool = get_pool().await;
    // clean DB, so only new message get processed
    sqlx::query!("DELETE FROM messages").execute(&pool).await?;
    let pool2 = pool.clone();

    let (tx, rx): (Sender<MessageEntity>, Receiver<MessageEntity>) = mpsc::channel();

    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen(QUEUE_NAME).await?;
    let messenger = Messenger::new(pool.clone());
    let process_fn = move |entity: &MessageEntity| {
        if let Err(_) = tx.send(entity.clone()) {
            println!("ERROR sending from process_fn()");
        }
    };
    let _ = tokio::spawn(async move {
        for _ in 0..3 {
            let _notification = listener.recv().await;
            messenger
                .process_next(&process_fn)
                .await
                .expect("Failed to process message");
        }
    });

    let messenger = Messenger::new(pool2);
    for i in 0..3 {
        let payload = CustomPayload { number: i };
        let msg = Message {
            payload: json!(payload),
        };
        messenger.add(msg).await.unwrap();
    }

    let mut result: Vec<i64> = vec![];
    for _ in 0..3 {
        let msg = rx.recv().unwrap();
        println!("recv(): {:?}", msg);
        result.push(msg.payload["number"].as_i64().unwrap())
    }

    let expected = vec![0, 1, 2];
    assert_eq!(expected, result);

    Ok(())
}

async fn get_pool() -> PgPool {
    let conn_str = std::env::var("DATABASE_URL").expect("Env Var DATABASE_URL must be set");
    PgPool::connect(&conn_str).await.unwrap()
}
