use crate::cache::AsyncRedisOperation;
use crate::time::INDEX_SUFFIX;
use crate::results::TimeIndexBaseInfo;

/// 异步方法：从redis缓存当中获取最后一条股票实时信息
pub async fn get_last_index_info_from_redis(redis_ope: &mut AsyncRedisOperation, ts_code: &String) -> Option<TimeIndexBaseInfo> {
    let mut redis_key = String::from(ts_code) + INDEX_SUFFIX;
    if redis_ope.exists(redis_key).await {
        redis_key = String::from(ts_code);
        redis_key = redis_key + INDEX_SUFFIX;
        let length = redis_ope.str_length::<String>(redis_key).await;
        // FIXME -- 此处写死了一个值，似乎单条的信息不会超过800吧
        let mut start = length - 800;
        // FIXME -- 此处有一个redis模块(依赖的redis模块，而不是redis服务器)的BUG：如果get_range的start的index正好位于中文字符串的中间，就不能成功返回数据了，此处修正一下
        // FIXME -- 如果start_index小于150，直接到0，这样能够避免这个问题吧，毕竟中文只在开头有
        if start < 150 {
            start = 0;
        }
        redis_key = String::from(ts_code);
        redis_key = redis_key + INDEX_SUFFIX;
        let ret_str = redis_ope.get_range::<String, String>(redis_key, start, length).await.unwrap();
        if ret_str.is_empty() {
            return None;
        }
        // 处理一下字符串，获取到最新的实时信息
        let temp_infos: Vec<&str> = ret_str.split('~').collect();
        if temp_infos.len() < 2 {
            return None;
        }
        if !temp_infos.is_empty() {
            let last_info_str = String::from(*temp_infos.get(temp_infos.len() - 2).unwrap());
            Some(last_info_str.into())
        }
        else {
            None
        }
    }
    else {
        None
    }
}