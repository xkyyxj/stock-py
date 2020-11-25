use redis::{AsyncCommands, ToRedisArgs, FromRedisValue};
use redis::aio::Connection;


pub struct AsyncRedisOperation {
    connection: Connection,
}

impl AsyncRedisOperation {
    pub(crate) async fn new() -> Self {
        let client = crate::initialize::REDIS_POOL.get().unwrap();
        let connection = client.get_async_connection().await.unwrap();
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

    pub(crate) async fn exists(&mut self, key: String) -> bool {
        self.connection.exists(key).await.unwrap()
    }

    pub(crate) async fn append_str(&mut self, key: String, value: String){
        let _: () = self.connection.append(key, value).await.unwrap();
    }

    pub(crate) async fn delete<K>(&mut self, key: K)
        where K: ToRedisArgs + Sync + Send {
        let _: () = self.connection.del(key).await.unwrap();
    }

    pub(crate) async fn str_length<K>(&mut self, key: K) -> isize
        where K: ToRedisArgs + Sync + Send {
        self.connection.strlen(key).await.unwrap()
    }

    pub(crate) async fn get_range<K, RV>(&mut self, key: K, start: isize, end: isize) -> Option<RV>
        where K: ToRedisArgs + Sync + Send,
              RV: FromRedisValue {
        match self.connection.getrange(key, start, end).await {
            Ok(val) => {
                Some(val)
            },
            Err(_) => {
                None
            }
        }
    }
}

