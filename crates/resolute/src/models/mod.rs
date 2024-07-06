#[cfg(all(feature = "db", feature = "models_v1"))]
pub mod v1;
pub mod v2;

pub use v2::*;
