use crate::results::DBResult;
use sqlx::query::Query;
use sqlx::{MySql, Row};
use sqlx::mysql::{MySqlArguments, MySqlRow};
use crate::sql;

/// 箱体震荡的股票
pub struct BoxStyle {
    pub(crate) pk_boxstyle: String,
    pub(crate) ts_code: String,             // 股票编码
    pub(crate) in_date: String,             // 进入榜单时间
    pub(crate) in_price: f64,               // 进入榜单时间的价格
    pub(crate) last_days: i64,              // 箱体已经持续的时间
    pub(crate) finished: String,            // 箱体震荡是否已经结束
    pub(crate) box_min_price: f64,          // 箱体之内的最低价格
}

impl DBResult for BoxStyle {
    fn new() -> Self {
        BoxStyle {
            pk_boxstyle: "".to_string(),
            ts_code: "".to_string(),
            in_date: "".to_string(),
            in_price: 0.0,
            last_days: 0,
            finished: "".to_string(),
            box_min_price: 0.0
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        sqlx::query("insert into box_style(ts_code, in_date, \
        in_price, last_days, finished, box_min_price) values(?,?,?,?,?,?)")
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        query = query.bind(&self.ts_code);
        query = query.bind(&self.in_date);
        query = query.bind(self.in_price);
        query = query.bind(self.last_days);
        query = query.bind(&self.finished);
        query.bind(self.box_min_price)
    }

    fn query(where_part: Option<String>) -> Vec<Box<Self>> {
        let mut final_sql = String::from("select * from box_style ");
        final_sql = super::process_where_part(final_sql, where_part);

        let mut final_rst = Vec::<Box<BoxStyle>>::new();
        sql::common_query(final_sql.as_ref(), |rows| {
            for row in rows {
                final_rst.push(Box::<Self>::new(process_single_row_for_box_style(row)));
            }
        });
        final_rst
    }
}

fn process_single_row_for_box_style(row: &MySqlRow) -> BoxStyle {
    let mut temp_rst = BoxStyle::new();
    let mut temp_str: String = row.get("ts_code");
    temp_rst.ts_code = temp_str;
    temp_rst.in_date = row.get("his_down_price");
    temp_rst.in_price = row.get("in_price");
    temp_rst.last_days = row.get("last_days");
    temp_rst.finished = row.get("finished");
    temp_rst.box_min_price = row.get("box_min_price");
    temp_rst
}