mod async_cache;
mod async_index_ope;

use redis::Commands;
pub use async_cache::AsyncRedisOperation;
pub use async_index_ope::{ get_last_index_info_from_redis, get_num_last_index_info_redis };

pub fn set(_key: &str, _content: &str) {
    // let client = crate::initialize::REDIS_POOL.get().unwrap();
    // let mut con = client.get_connection().unwrap();
    // con.set(key, content);

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut _con = client.get_connection().unwrap();
    let _: () = _con.set("123", 23).unwrap();
}
