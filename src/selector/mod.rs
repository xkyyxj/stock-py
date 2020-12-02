mod ema_select;
mod rst_process;
mod final_select;
mod history_down_select;

use chrono::{DateTime, Local};

static ALWAYS_DOWN: i32 = -1;       // 一直下降
static DOWN_THEN_UP: i32 = 0;       // 经历过拐点(先下降后上升)
static UP_THEN_DOWN: i32 = 2;       // 先上升后下降
static ALWAYS_UP: i32 = 2;          // 一直上涨
static SINGLE_PRICE: i32 = 3;       // 一直一个价格
static WAVE: i32 = 4;               // 反复波动

pub(crate) static SHORT_TYPE: i32 = 1;  // 短期选择结果
pub(crate) static LONG_TYPE: i32 = 2;   // 长期选择结果
pub(crate) static FINAL_TYPE: i32 = 4;    // 最终待选股票-要插入到wait_select表当中的数据

pub struct SingleCommonRst {
    pub ts_code: String,
    pub ts_name: String,
    pub curr_price: f64,
    pub level: i64,              // 评分：0-100分
    pub source: String,          // 来源系统，通过ema选定还是什么其他指标
    pub level_pct: f64,          // 得分的百分比
    pub line_style: i32,         // 分时线形态：-1 一直下降；0 经历过拐点(先下降后上升)；1 先上升后下降；2 一直上涨；3 一直一个价;4 反复波动
    pub rst_style: i32,          // 结果形态：短线、长线、都适用
}

pub struct CommonSelectRst {
    pub select_rst: Vec<SingleCommonRst>,
    pub ts: DateTime<Local>,
}

impl Clone for SingleCommonRst {
    fn clone(&self) -> Self {
        SingleCommonRst {
            ts_code: String::from(&self.ts_code),
            ts_name: String::from(&self.ts_name),
            curr_price: self.curr_price,
            level: self.level,
            source: String::from(&self.source),
            level_pct: self.level_pct,
            line_style: self.line_style,
            rst_style: self.rst_style,
        }
    }
}

impl Clone for CommonSelectRst {
    fn clone(&self) -> Self {
        let mut vec: Vec<SingleCommonRst> = vec![];
        for item in &self.select_rst {
            vec.push(item.clone());
        }
        CommonSelectRst {
            select_rst: vec,
            ts: self.ts.clone()
        }
    }
}

impl CommonSelectRst {
    pub(crate) fn new() -> Self {
        CommonSelectRst { select_rst: vec![], ts: Local::now() }
    }

    // FIXME -- 此处各个字段的处理是不是还需要精细化一些
    pub(crate) fn add_selected(&mut self, info : SingleCommonRst) {
        let mut contains = false;
        for item in &mut self.select_rst {
            if item.ts_code == info.ts_code {
                // 如果已经存在了，那么就直接update一下就好
                item.line_style = info.line_style;
                item.level = info.level;
                item.curr_price = info.curr_price;
                item.rst_style = item.rst_style & info.rst_style;
                // TODO source如何处理，得看下
                contains = true;
            }
        }
        if !contains {
            self.select_rst.push(info);
        }
    }

    /// 合并结果用于多个不同的选择策略的合并，蒋选择结果合并到最终结果当中需要用到append方法
    /// 两个结果的合并，重复的结果得分的简单相加，只在一方存在的结果添加到最终结果集里面
    pub(crate) fn merge(&mut self, other: &CommonSelectRst) {
        let mut only_one = Vec::<SingleCommonRst>::new();
        for other_item in &other.select_rst {
            let mut contain = false;
            for self_item in &mut self.select_rst {
                if self_item.ts_code == other_item.ts_code {
                    self_item.level = self_item.level + other_item.level;
                    if self_item.level > 100 {
                        self_item.level = 100;
                    }
                    // 更正一下结果适用范围（短线、长线？）
                    self_item.rst_style = self_item.rst_style & other_item.rst_style;
                    contain = true;
                    break;
                }
            }
            if !contain {
                only_one.push(other_item.clone());
            }
        }
        if !only_one.is_empty() {
            self.select_rst.append(&mut only_one);
        }
        self.ts = Local::now();
    }

    /// 蒋某次选择结果汇总到最终结果中来
    /// @return 返回所有在这一个append当中可买入的股票
    pub(crate) fn append(&mut self, other: &CommonSelectRst) -> Vec<String> {
        let mut ret_rst = Vec::<String>::new();
        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        let short_time_buy_level = config.short_buy_level;
        let mut only_one = Vec::<SingleCommonRst>::new();
        for other_item in &other.select_rst {
            let mut contain = false;
            for self_item in &mut self.select_rst {
                if self_item.ts_code == other_item.ts_code {
                    // 小于short_time_buy_level就等于没买过，
                    // 原先没买过，但是新的选择结果其买入等级level更高，那么就将结果值赋成更高的level
                    if self_item.level < short_time_buy_level && self_item.level < other_item.level {
                        self_item.level = other_item.level;
                        self_item.curr_price = other_item.curr_price;
                        self_item.source = other_item.source.clone();
                        ret_rst.push(String::from(&self_item.ts_code));
                    }
                    // 已经买入过了
                    else if self_item.level >= short_time_buy_level {
                        if self_item.level < other_item.level {
                            self_item.level = other_item.level;
                        }
                    }
                    // 对于self_item.level > other_item.level，一概不处理

                    // 更正一下结果适用范围（短线、长线？）
                    self_item.rst_style = self_item.rst_style & other_item.rst_style;
                    self_item.line_style = other_item.line_style;
                    contain = true;
                    break;
                }
            }
            if !contain {
                let temp_val = other_item.clone();
                if temp_val.level >= short_time_buy_level {
                    ret_rst.push(String::from(&temp_val.ts_code));
                }
                only_one.push(temp_val);
            }
        }
        if !only_one.is_empty() {
            self.select_rst.append(&mut only_one);
        }
        self.ts = Local::now();
        ret_rst
    }
}
