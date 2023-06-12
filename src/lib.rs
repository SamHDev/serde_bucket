

#[doc = include_str!("../README.md")]

mod node;
mod bucket;
mod debug;
mod de;

pub use bucket::*;

#[cfg(feature = "error")]
mod error;
mod ser;

// mod segment;

#[cfg(feature = "error")]
pub use error::*;

#[cfg(feature="deserializer")]
pub use de::*;