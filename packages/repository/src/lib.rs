// Root-level repositories module matches directory packages/repositories/src/repositories
pub mod repositories;

// Back-compat within this crate for code that used `crate::shared::data::repositories`
pub mod shared {
    pub mod data {
        pub use crate::repositories;
    }
}