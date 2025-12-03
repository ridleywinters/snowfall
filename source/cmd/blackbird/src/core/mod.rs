pub mod prelude {}

pub mod internal {
    pub use super::prelude::*;
}

mod local_storage;
pub use local_storage::LocalStorage;
