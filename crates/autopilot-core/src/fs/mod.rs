use std::sync::OnceLock;
pub mod get;
pub use get::*;
pub mod set;
pub use set::*;

pub static CONFIG_PATH: OnceLock<String> = OnceLock::new();
