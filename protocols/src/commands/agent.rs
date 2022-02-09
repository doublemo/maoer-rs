use std::fmt;

/// 网关命令
#[derive(Debug)]
pub enum Agent {
    /// 加密握手
    Handshake = 1000,

    /// 数据通道握手
    Datachannel,

    /// 心跳
    Heartbeater,

    /// 踢掉玩家
    KickedOut,

    /// 广播消息
    Broadcast,
}

impl Agent {
    /// 返回i32值
    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
