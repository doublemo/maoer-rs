#![warn(dead_code, unused_macros)]

use std::{fmt, error};

mod macros;

#[allow(unused)]
/// 自定义错误Result
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    pub code:i32,
    pub message:String
}

impl Error {
    pub fn new(code:i32, message:String) -> Self {
        Error{code, message}
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        From::from(Error{code:0, message:err})
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        From::from(Error{code:0, message:String::from(err)})
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}> {}", self.code, self.message)
    }
}


impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub fn make_errcode(id:i32, seq:u32, extends:Option<u32>) -> i32 {
    let ext = match extends {
        Some(i) => i,
        None => 0
    };

    id << (16 as usize) | (seq as i32) << (3 as usize) | (ext as i32)
}

pub fn paser_errcode(code:i32)-> (i32, u32, u32) {
    let extends = (code << (29 as usize)) >> (29 as usize);
    let sequence = (code << (16 as usize)) >> (19 as usize);
    let kit = code >> 16;

    (kit,sequence as u32, extends as u32)
}