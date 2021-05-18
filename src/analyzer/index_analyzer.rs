use crate::results::{HistoryDown, DBResult, QueryInfo};

pub async fn index_analyzer() {

}

async fn history_down_ana() {
    // 第零步：查询位于history_down当中的股票
    let query_info = Default::default();
    let _history_down_vos = HistoryDown::query(&query_info);
    // 第一步：查看一下是否当前价格已经突破到了前一天的最高价
    // 第三步：输出数据到选定表格当中
}