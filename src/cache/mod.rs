use redis::Commands;

pub fn set(key: &str, content: &str) {
    let client = super::REDIS_POOL.get().unwrap();
    let mut con = client.get_connection().unwrap();
    con.set(key, content);
}
