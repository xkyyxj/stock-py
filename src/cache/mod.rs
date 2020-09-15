

pub fn set(_key: &str, _content: &str) {
    // let client = crate::initialize::REDIS_POOL.get().unwrap();
    // let mut con = client.get_connection().unwrap();
    // con.set(key, content);

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let _con = client.get_connection().unwrap();
    // con.set("123", 23);
}
