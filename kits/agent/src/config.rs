use std::sync::RwLock;
use config::{Config, ConfigError, File};
use std::env;
use std::path::PathBuf;
use tracing;
use serde::Deserialize;

lazy_static::lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new(Config::default());
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Socket {
    pub backlog: usize,
    pub addr:String,
    pub port: usize
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Websocket {
    pub addr:String,
    pub port: usize
}


/// 网关配置文件
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Configuration{
    /// 调试模式
    pub debug: bool,

    /// 日志输出等级, 支持 trace, debug, info, warn, error
    pub log_level:String,

    /// socket网络连接设置
    pub socket: Socket,

    /// websocket连接设置
    pub websocket:Option<Websocket>,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let s = SETTINGS.read().unwrap().clone();
        let mut config:Configuration = s.try_into()?;
        if config.websocket.is_none() {
            config.websocket = Some(Websocket::default());
        }
        Ok(config)
    }
}

impl Default for Websocket {
    fn default() -> Self {
        Websocket {
            addr:String::from("0.0.0.0"),
            port: 8089,
        }
    }
}

/// 初始化配置文件
pub fn init(config_file:&str) {  
    let path = match env::current_dir() {
        Ok(mut path) => {
            path.push(config_file);
            path
        }
        Err(_) => PathBuf::from(config_file)
    };

    if !path.exists() {
        panic!("config file is not found.")
    }
    
    if let Some(file) = path.as_path().to_str() {
        let mut config = SETTINGS.write().unwrap();
        config.merge(File::with_name(file)).unwrap();
    } else {
        panic!("config file is not found. {}", config_file)
    }
}

/// 显示配置文件信息
pub fn show() {
    let config = Configuration::new().unwrap();
    tracing::info!("----------------- MAOER ----------------------");
    tracing::info!(" debug: {}", config.debug);
    tracing::info!(" log_level: {}", config.log_level);
    tracing::info!(" socket.backlog: {}", config.socket.backlog);
    tracing::info!(" socket.backlog: {}", config.socket.backlog);
    tracing::info!(" socket.addr: {}:{}", config.socket.addr, config.socket.port);
    if let Some(w) = config.websocket {
        tracing::info!(" websocket.addr: {}:{}", w.addr, w.port);
    }

    tracing::info!("----------------------------------------------");
}
