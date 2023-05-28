## Serde Buckets

'Buckets' for [Serde] Deserialisation

[![crates.io badge](https://img.shields.io/crates/v/serde_bucket.svg?style=for-the-badge)](https://crates.io/crates/tycho)
[![docs.rs badge](https://img.shields.io/docsrs/serde_bucket.svg?style=for-the-badge&color=blue)](https://docs.rs/tycho)
[![Downloads badge](https://img.shields.io/crates/d/serde_bucket.svg?style=for-the-badge)](https://crates.io/crates/tycho)

---

The [`Bucket`] stores data (and it's type) according to the serde data model,
allowing for format analogous representation of any self-describing type.

This type is intended for use in the deserialisation process, where the structure is
not known until a given field is parsed. This type can store that data without
copying or taking ownership of heap allocated types (unlike [`serde_value`](serde_value)).

Under the hood, the `Bucket` type is a `vec` of 'nodes' that depict the type, it's value
and other attributes. This makes it easier to work with and removes some of the
heap allocation that you find with recursive data structures.

> This is a crate created for *my* projects, and as such is unlikely to be maintained.
It might be worth saying *'bucket'* and use [`serde_value`] instead.

[`Bucket`]: https://docs.rs/serde_bucket/0.1.1/serde_bucket/struct.Bucket.html
[`serde`]: https://serde.rs
[`serde_value`]: https://docs.rs/serde-value/0.7.0/serde_value/


### Example
The following examples use `serde_json` as the format 

```rust
use serde_bucket::Bucket;

// parse an input using your favourite serde library
// deserialise it into the `Bucket` type.
let input = r#"{"a": 10, "b": false}"#;
let mut  bucket: Bucket = serde_json::from_str(&input).unwrap();

// our example structure
#[derive(Deserialize)]
struct Example {
    a: u8,
    b: bool
}

// use `deserialize_into` to "deserialise into" a given type.
// the error type (in this `serde_json::Error`) must implement `serde::de::Error`
let value = bucket.deserialize_into::<Example, serde_json::Error>().unwrap();
assert_eq!(value, Example { a: 10, b: false });
```


#### Thanks
Merci Buckets for stopping by `<3` - here is [My Twitter](https://samh.dev/twitter) if you want to say hello