use futures::prelude::*;
use redis::AsyncCommands;
use redis::aio::Connection;

pub async fn get_connection(&client: redis::Client) {
    client.get_async_connection().await.unwrap()
}

pub async fn set(key: &str, value: &str, &mut conn: Connection) {
    conn.set(key, value);
}

pub async fn get(key: &str, &mut conn: Connection) {
    conn.get(key).unwrap()
}