//! 'Buckets' for [Serde](serde) Deserialisation
//!
//! The [`Bucket`](crate::Bucket) stores data (and it's type) according to the serde data model,
//! allowing for format analogous representation of any self-describing type.
//!
//! This type is intended for use in the deserialisation process, where the structure is
//! not known until a given field is parsed. This type can store that data without
//! copying or taking ownership of heap allocated types (unlike [`serde_value`](serde_value)).
//!
//! Under the hood, the `Bucket` type is a `vec` of 'nodes' that depict the type, it's value
//! and other attributes. This makes it easier to work with and removes some of the
//! heap allocation that you find with recursive data structures.
//!
//! This is a crate created for *my* projects, and as such is unlikely to be maintained.
//! It might be worth saying *'bucket'* and use [`serde_value`](serde_value) instead.
//!
//! #### Example
//! The following examples use `serde_json` as it's source
//!
//! ```
//! use serde_bucket::Bucket;
//!
//! // parse an input using your favourite serde library
//! // deserialise it into the `Bucket` type.
//! let input = r#"{"a": 10, "b": false}"#;
//! let mut  bucket: Bucket = serde_json::from_str(&input).unwrap();
//!
//! // our example structure
//! #[derive(Deserialize)]
//! struct Example {
//!     a: u8,
//!     b: bool
//! }
//!
//! // use `deserialize_into` to "deserialise into" a given type.
//! // the error type (in this `serde_json::Error`) must implement `serde::de::Error`
//! let value = bucket.deserialize_into::<Example, serde_json::Error>().unwrap();
//! assert_eq!(value, Example { a: 10, b: false });
//! ```
//!

mod node;
mod bucket;
mod debug;
mod de;

pub use bucket::*;

#[cfg(feature = "error")]
mod error;

#[cfg(feature = "error")]
pub use error::*;