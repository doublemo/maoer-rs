/// 错误码定义
/// 
/// #Examples
/// 
/// ```
/// # use maoer_agent::errors::Kind;
/// # use maoer_cores::error::{paser_errcode};
/// # fn main() {
/// let kd = Kind::CommandInvalid;
/// let err = kd.as_error();
/// println!("{} {} {:?}",  err, err.code, paser_errcode(err.code));
/// # }
/// ```
use maoer_cores::make_errors;
use maoer_protocols::kits::Kits;

make_errors! (
    Kind, 
    Kits::Agent.to_i32(),
    CommandInvalid => "Invalid call command code in request",
    InvalidProtoVersion => "Invalid  call proto version in request",
    InvalidSEQID  => "Invalid call sequence id in request",
    InternalServer => "Internal Server Error",
);
