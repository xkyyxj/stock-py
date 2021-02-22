use redis::{AsyncCommands, ToRedisArgs, FromRedisValue, ConnectionLike};
use redis::aio::Connection;
use chrono::Local;


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

    async fn check_connection(&mut self) {
        if let Err(err) =  self.connection.exists::<String, bool>(String::from("~")).await {
            println!("check connection err: {}", err);
            self.reconnect().await;
        }
    }

    pub async fn reconnect(&mut self) {
        println!("reconnect time is {}", Local::now());
        let client = crate::initialize::REDIS_POOL.get().unwrap();
        match client.get_async_connection().await {
            Ok(connection) => {
                self.connection = connection;
            },
            Err(err) => {
                println!("reconnect err!! {}， time is {}", err, Local::now());
                let redis_info = crate::initialize::CONFIG_INFO.get().unwrap().redis_info.as_str();
                if let Ok(val) = redis::Client::open(redis_info) {
                    crate::initialize::REDIS_POOL.set(val).unwrap();
                };
            }
        }

    }

    pub(crate) async fn set<K, V>(&mut self, key: K, value: V)
        where K: ToRedisArgs + Sync + Send,
              V: ToRedisArgs + Sync + Send {
        self.check_connection().await;
        let _: () = self.connection.set(key, value).await.unwrap();
    }

    pub(crate) async fn get<K, RV>(&mut self, key: K) -> Option<RV>
        where K: ToRedisArgs + Sync + Send,
              RV: FromRedisValue {
        self.check_connection().await;
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
        self.check_connection().await;
        self.connection.exists(key).await.unwrap()
    }

    pub(crate) async fn append_str(&mut self, key: String, value: String){
        self.check_connection().await;
        let _: () = self.connection.append(key, value).await.unwrap();
    }

    pub(crate) async fn delete<K>(&mut self, key: K)
        where K: ToRedisArgs + Sync + Send {
        self.check_connection().await;
        let _: () = self.connection.del(key).await.unwrap();
    }

    pub(crate) async fn str_length<K>(&mut self, key: K) -> isize
        where K: ToRedisArgs + Sync + Send {
        self.check_connection().await;
        self.connection.strlen(key).await.unwrap()
    }

    pub(crate) async fn get_range<K, RV>(&mut self, key: K, start: isize, end: isize) -> Option<RV>
        where K: ToRedisArgs + Sync + Send,
              RV: FromRedisValue {
        self.check_connection().await;
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

