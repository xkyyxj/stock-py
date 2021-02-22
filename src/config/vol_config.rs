/// VOL上涨的判定周期
pub static VOL_EMA_LENGTH: u64 = 4;


#[derive(Debug)]
pub struct VoLConfig {
    vol_ema_length: u64,
}