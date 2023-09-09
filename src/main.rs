use std::time::Duration;

use pgqueue::sql::{listen, Message, MessageEntity, Messenger, Payload};
use serde_json::json;
use sqlx::PgPool;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    foo().await;
    sleep(Duration::from_millis(5000)).await;
}

async fn foo() {
    let pool = get_pool().await;
    let pool2 = pool.clone();

    tokio::spawn(async move {
        println!("started tokio spawn()");
        let process_fn = |entity: &MessageEntity| {
            println!("processFn(): message.id: {:?}", entity.id);
            println!("processFn(): message.payload: {:?}", entity.payload);
            Ok(())
        };
        match listen(pool, process_fn).await {
            Err(err) => println!("ERROR in listener, {:?}", err),
            _ => println!("listner should bloc"),
        }
    });

    // NOTIFY channel, with add() new message into channel
    let payload = Payload {
        version: 1,
        kind: "Command".to_owned(),
        message: "my new message notification".to_owned(),
    };
    let msg = Message {
        payload: json!(payload),
    };
    let sut = Messenger::new(pool2);
    sut.add(msg).await.unwrap();
}

async fn get_pool() -> PgPool {
    let conn_str = std::env::var("DATABASE_URL").expect("Env Var DATABASE_URL must be set");
    PgPool::connect(&conn_str).await.unwrap()
}
