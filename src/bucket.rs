use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use serde::{Deserialize, Deserializer};
use crate::de::{BucketDeserializer, BucketVisitor};
use crate::debug::debug_nodes;
use crate::node::BucketNode;

/// A Serde Bucket.
///
/// Stores Serde data model values for later deserialisation
///
/// `Bucket` implements `serde::Deserialize` in-order to parse data,
/// and has two methods to extract data:
/// - [`deserialize_into`] - takes ownership of the owned/heap values (via `mem::swap`)
/// and replaces them with null-like marker values.
/// - [`deserialize_into_clone`[ - clones the owned/heap values which allows for
/// multiple deserializations on the same bucket.
///
/// [`deserialize_into`]: Bucket::deserialize_into
/// [`deserialize_into_clone`]: Bucket::deserialize_into_clone
///
/// This type also implements `debug`, but the output is, well, not very good.
pub struct Bucket<'a> {
    pub(crate) inner: Vec<BucketNode<'a>>
}

impl<'a> Debug for Bucket<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        debug_nodes(&self.inner, f)
    }
}

/*

pub struct OwnedBucket {
    inner: Bucket<'static>
}

impl Debug for OwnedBucket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        debug_nodes(&self.inner.inner, f)
    }
}
*/

impl<'a> Bucket<'a> {
    /// deserialize into (clone method)
    ///
    /// This function deserializes into the given type, cloning owned data (Vec, String)
    /// - `T` as `Deserialize`
    /// - `E` as a `serde::de::Error`
    pub fn deserialize_into_clone<T: Deserialize<'a>, E: serde::de::Error>(&'a mut self) -> Result<T, E> {
        T::deserialize(&mut BucketDeserializer {
            buffer: &mut self.inner,
            cursor: 0,
            error: PhantomData::<E>::default(),
            clone: true,
        })
    }

    /// deserialize into (clone method)
    ///
    /// This function deserializes into the given type, taking/replacing owned data (Vec, String)
    /// - `T` as `Deserialize`
    /// - `E` as a `serde::de::Error`
    pub fn deserialize_into<T: Deserialize<'a>, E: serde::de::Error>(&'a mut self) -> Result<T, E> {
        T::deserialize(&mut BucketDeserializer {
            buffer: &mut self.inner,
            cursor: 0,
            error: PhantomData::<E>::default(),
            clone: false,
        })
    }

    #[cfg(feature="deserializer")]
    /// get a 'deserializer' for custom deserialization
    pub fn deserializer<E: serde::de::Error>(&'a mut self) -> BucketDeserializer<'a, E> {
        BucketDeserializer {
            buffer: &mut self.inner,
            cursor: 0,
            error: PhantomData::<E>::default(),
            clone: false,
        }
    }
}

impl<'de> Deserialize<'de> for Bucket<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let mut nodes = Vec::new();
        deserializer.deserialize_any(&mut BucketVisitor {
            target: &mut nodes,
            owned: false,
        })?;
        Ok(Self { inner: nodes })
    }
}

/*
impl OwnedBucket {
    pub fn deserialize_into<T: DeserializeOwned, E: serde::de::Error>(mut self) -> Result<T, E> {
        let mut deser = BucketDeserializer {
            buffer: &mut self.inner.inner,
            cursor: 0,
            error: PhantomData::<E>::default(),
            clone: false,
        };
        T::deserialize(deser)
    }
}*/