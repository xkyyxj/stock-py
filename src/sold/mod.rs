mod short_time_sold;
mod track_sold;
mod history_down_policy;

pub use track_sold::TrackSold;

pub struct SoldInfo {
    pub pk_buy_ope: i64,
    pub ope_pct: f64,
    pub can_sold: bool,
    pub sold_price: f64
}