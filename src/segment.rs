use std::marker::PhantomData;
use serde::Deserialize;
use crate::BucketDeserializer;
use crate::node::BucketNode;

fn skip_item(offset: &mut usize, slice: &mut [BucketNode]) {
    while offset >= &slice.len() {
        match slice[offset] {

        }
    }
}

pub struct BucketSegment<'r, 'a> {
    slice: &'a mut [BucketNode<'a>]
}

impl<'r, 'a> BucketSegment<'r, 'a> {
    pub fn try_seq_index(self, index: usize) -> Option<BucketSegment<'r, 'a>> {
        if let BucketNode::Seq(size) = self.slice.get(0)? {
            if index >= *size {
                return None;
            }


        }
    }


    /// deserialize into (clone method)
    ///
    /// This function deserializes into the given type, cloning owned data (Vec, String)
    /// - `T` as `Deserialize`
    /// - `E` as a `serde::de::Error`
    pub fn deserialize_into_clone<T: Deserialize<'a>, E: serde::de::Error>(&'a mut self) -> Result<T, E> {
        T::deserialize(&mut BucketDeserializer {
            buffer: &mut self.slice,
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
            buffer: &mut self.slice,
            cursor: 0,
            error: PhantomData::<E>::default(),
            clone: false,
        })
    }

    #[cfg(feature="deserializer")]
    /// get a 'deserializer' for custom deserialization
    pub fn deserializer<E: serde::de::Error>(&'a mut self) -> BucketDeserializer<'a, E> {
        BucketDeserializer {
            buffer: &mut self.slice,
            cursor: 0,
            error: PhantomData::<E>::default(),
            clone: false,
        }
    }
}