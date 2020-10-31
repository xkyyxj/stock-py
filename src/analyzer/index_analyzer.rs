use crate::results::{HistoryDown, DBResult};

pub async fn index_analyzer() {

}

async fn history_down_ana() {
    // 第零步：查询位于history_down当中的股票
    let history_down_vos = HistoryDown::query(Option::None);
    // 第一步：查看一下是否当前价格已经突破到了前一天的最高价
    // 第三步：输出数据到选定表格当中
}