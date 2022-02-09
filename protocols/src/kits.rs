#![warn(missing_docs, unused_variables, dead_code)]

use std::fmt;

/// 服务全局唯一编号
#[derive(Debug)]
pub enum Kits {
    /// 网关
    Agent = 1, 

    /// sfu
    SFU,

    /// 服务
    AUTH,
}

impl Kits {
    /// 返回i32值
    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for Kits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

