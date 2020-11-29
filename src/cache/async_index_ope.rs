use crate::cache::AsyncRedisOperation;
use crate::time::INDEX_SUFFIX;
use crate::results::TimeIndexBaseInfo;

/// 异步方法：从redis缓存当中获取最后一条股票实时信息
pub async fn get_last_index_info_from_redis(redis_ope: &mut AsyncRedisOperation, ts_code: &String) -> Option<TimeIndexBaseInfo> {
    if let Some(mut val) = get_num_last_index_info_redis(redis_ope, ts_code, 1).await {
        return Some(val.remove(0));
    }
    None
}

/// 异步方法：从redis缓存当中获取最后几条实时信息，如果达不到传入参数要求的数据量，返回尽量多的
/// Important!!!! -- 返回的Vec当中index越小越新
pub async fn get_num_last_index_info_redis(redis_ope: &mut AsyncRedisOperation, ts_code: &String, num: i64)
    -> Option<Vec<TimeIndexBaseInfo>> {
    let mut redis_key = String::from(ts_code) + INDEX_SUFFIX;
    if redis_ope.exists(redis_key).await {
        redis_key = String::from(ts_code);
        redis_key = redis_key + INDEX_SUFFIX;
        let length = redis_ope.str_length::<String>(redis_key).await;
        // FIXME -- 此处写死了一个值，似乎单条的信息不会超过400吧
        // 多获取一个，避免头部问题（头部不是TimeIndexBaseInfo结构）
        let mut start = length - 400 * (num as isize + 1);
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
            let mut base_info_vec = Vec::<TimeIndexBaseInfo>::new();
            for i in 0..num {
                let curr_index = temp_infos.len() - 2 - i as usize;
                // 上面从redis当中多获取了一个，所以这个地方的话如果是小于1的话，就不要了
                if curr_index < 1 {
                    return Some(base_info_vec);
                }

                // 特殊处理一下一种情况：当上面的变量start为0的时候，curr_index为1的话
                // 此时*temp_infos.get(curr_index).unwrap()实际上是股票编码，所以会导致报错
                if start == 0 && curr_index < 2 {
                    return Some(base_info_vec);
                }
                let last_info_str = String::from(*temp_infos.get(curr_index).unwrap());
                base_info_vec.push(last_info_str.into());
            }
            Some(base_info_vec)
        }
        else {
            None
        }
    }
    else {
        None
    }
}