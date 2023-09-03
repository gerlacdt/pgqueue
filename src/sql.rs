use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub fn hello() {
    println!("Hello World!");
}

#[derive(Clone, Debug, PartialEq, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "message_status", rename_all = "lowercase")]
pub enum MessageStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug)]
pub struct MessageEntity {
    pub id: i32,
    pub status: MessageStatus,
    pub payload: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub struct Message {
    pub payload: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
pub struct Payload {
    version: i32,
    kind: String,
    message: String,
}

struct Messenger {
    pool: PgPool,
}

impl Messenger {
    pub fn new(pool: PgPool) -> Self {
        Messenger { pool }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<MessageEntity, sqlx::Error> {
        let record = sqlx::query!(
            r#"
            SELECT id, status AS "status!: MessageStatus", payload, created_at, updated_at
            FROM messages
            WHERE id = $1;
"#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        let entity = MessageEntity {
            id: record.id,
            status: record.status,
            payload: record.payload.into(),
            created_at: record.created_at,
            updated_at: record.updated_at,
        };

        Ok(entity)
    }

    pub async fn add(&self, message: Message) -> Result<MessageEntity, sqlx::Error> {
        let record = sqlx::query!(
            "INSERT INTO messages(status, payload) VALUES ($1, $2)
RETURNING id, status AS \"status!: MessageStatus\", payload, created_at, updated_at;",
            MessageStatus::Pending as MessageStatus,
            message.payload
        )
        .fetch_one(&self.pool)
        .await?;

        let entity = MessageEntity {
            id: record.id,
            status: MessageStatus::Pending,
            payload: record.payload.into(),
            created_at: record.created_at,
            updated_at: record.updated_at,
        };

        Ok(entity)
    }

    pub async fn fetch_next(&self) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query!(
            r#"select id, status AS "status!: MessageStatus", payload, created_at, updated_at
             FROM messages where status = 'pending'
             ORDER BY id ASC
             FOR UPDATE SKIP LOCKED
             LIMIT 1;
"#
        )
        .fetch_one(&mut *transaction)
        .await?;

        let entity = MessageEntity {
            id: result.id,
            status: result.status,
            payload: result.payload.into(),
            created_at: result.created_at,
            updated_at: result.updated_at,
        };

        sqlx::query!(
            r#"
            UPDATE messages
            SET status = 'completed'
            where id = $1
"#,
            entity.id
        )
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn hello_test() {
        hello();
        assert!(true);
    }

    #[tokio::test]
    async fn add_test() {
        let pool = get_pool().await;
        setup_db(pool.clone()).await.unwrap();
        let sut = Messenger::new(pool);
        let payload = Payload {
            version: 1,
            kind: "Command".to_owned(),
            message: "Hello World from message".to_owned(),
        };
        let msg = Message {
            payload: json!(payload),
        };
        let actual = sut.add(msg).await.unwrap();

        let now = Utc::now();
        assert!(actual.id >= 1, "id must be set and positive");
        assert!(actual.created_at <= now);
        assert!(actual.updated_at <= now);
        assert_eq!(json!(payload).to_string(), actual.payload.to_string())
    }

    #[tokio::test]
    async fn fetch_next_test() {
        let pool = get_pool().await;
        setup_db(pool.clone()).await.unwrap();
        let sut = Messenger::new(pool);
        let payload = Payload {
            version: 1,
            kind: "Command".to_owned(),
            message: "fetch_next message".to_owned(),
        };
        let msg = Message {
            payload: json!(payload),
        };
        let saved_message = sut.add(msg).await.unwrap();

        sut.fetch_next().await.unwrap();

        let actual = sut.find_by_id(saved_message.id).await.unwrap();

        println!("{:?}", actual);

        assert_eq!(MessageStatus::Completed, actual.status);
        assert_eq!(saved_message.id, actual.id);
    }

    async fn get_pool() -> PgPool {
        let conn_str = std::env::var("DATABASE_URL").expect("Env Var DATABASE_URL must be set");
        let pool = PgPool::connect(&conn_str).await.unwrap();
        pool
    }

    async fn setup_db(pool: PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM messages").execute(&pool).await?;
        Ok(())
    }
}
