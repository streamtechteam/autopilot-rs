mod error;
mod jsonc;
mod paths;
mod store;

pub use error::{FsError, Result};
pub use jsonc::jsonc_parse;
pub use paths::{Paths, init, paths};
pub use store::{JsonFileStore, RecordId};

//  TODO
// need to use better names for this module, in general needs refactoring
