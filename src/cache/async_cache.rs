use redis::{AsyncCommands, ToRedisArgs, FromRedisValue};
use redis::aio::Connection;
use futures::executor;

pub struct AsyncRedisOperation {
    connection: Connection,
}

impl AsyncRedisOperation {
    pub(crate) async fn new() -> Self {
        let client = crate::initialize::REDIS_POOL.get().unwrap();
        let mut connection = client.get_async_connection().await.unwrap();
        AsyncRedisOperation {
            connection
        }
    }

    pub(crate) async fn set<K, V>(&mut self, key: K, value: V)
        where K: ToRedisArgs + Sync + Send,
              V: ToRedisArgs + Sync + Send {
        let _: () = self.connection.set(key, value).await.unwrap();
    }

    pub(crate) async fn get<K, RV>(&mut self, key: K) -> Option<RV>
        where K: ToRedisArgs + Sync + Send,
              RV: FromRedisValue {
        match self.connection.get(key).await {
            Ok(val) => {
                Some(val)
            },
            Err(_) => {
                None
            }
        }
    }

    pub(crate) async fn delete<K>(&mut self, key: K)
        where K: ToRedisArgs + Sync + Send {
        let _: () = self.connection.del(key).await.unwrap();
    }
}

