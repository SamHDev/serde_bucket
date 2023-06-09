use std::fmt::Formatter;
use std::marker::PhantomData;
use std::mem;
use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserializer, forward_to_deserialize_any};
use crate::node::BucketNode;

pub struct BucketDeserializer<'de, E> where E: serde::de::Error {
    pub(crate) buffer: &'de mut [BucketNode<'de>],
    pub(crate) cursor: usize,
    pub(crate) error: PhantomData<E>,
    pub(crate) clone: bool,
}


impl<'x, 'de, E> Deserializer<'de> for &'x mut BucketDeserializer<'de, E> where E: serde::de::Error {
    type Error = E;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        let node=  &mut self.buffer[self.cursor];
        self.cursor += 1;
        match node {
            BucketNode::Consumed => return Err(E::custom("value as already been consumed")),
            BucketNode::Unsized => return Err(E::custom("invalid value - no size data")),

            BucketNode::Unit => visitor.visit_unit(),
            BucketNode::None => visitor.visit_none(),

            BucketNode::Bool(val) => visitor.visit_bool(*val),
            BucketNode::U8(val) => visitor.visit_u8(*val),
            BucketNode::I8(val) => visitor.visit_i8(*val),
            BucketNode::U16(val) => visitor.visit_u16(*val),
            BucketNode::I16(val) => visitor.visit_i16(*val),
            BucketNode::U32(val) => visitor.visit_u32(*val),
            BucketNode::I32(val) => visitor.visit_i32(*val),
            BucketNode::U64(val) => visitor.visit_u64(*val),
            BucketNode::I64(val) => visitor.visit_i64(*val),
            BucketNode::U128(val) => visitor.visit_u128(*val),
            BucketNode::I128(val) => visitor.visit_i128(*val),
            BucketNode::F32(val) => visitor.visit_f32(*val),
            BucketNode::F64(val) => visitor.visit_f64(*val),
            BucketNode::Char(val) => visitor.visit_char(*val),

            BucketNode::Bytes(bytes) if self.clone => {
                visitor.visit_byte_buf(bytes.clone())
            },
            val @ BucketNode::Bytes(_) => {
                let mut drain = BucketNode::Consumed;
                mem::swap(val, &mut drain);
                let BucketNode::Bytes(bytes) = drain else {
                    unreachable!()
                };
                visitor.visit_byte_buf(bytes)
            }
            BucketNode::BytesRef(val) => visitor.visit_borrowed_bytes(val),

            BucketNode::String(string) if self.clone => {
                visitor.visit_string(string.clone())
            },
            val @ BucketNode::String(_) => {

                let mut drain = BucketNode::Consumed;
                mem::swap(val, &mut drain);
                let BucketNode::String(string) = drain else {
                    unreachable!()
                };

                visitor.visit_string(string)
            },
            BucketNode::StringRef(val) => visitor.visit_borrowed_str(val),

            BucketNode::Some => visitor.visit_some(self),

            BucketNode::NewType => visitor.visit_newtype_struct(self),
            BucketNode::Seq(size) => visitor.visit_seq(BucketSeqDeserializer {
                size: *size,
                bucket: self,
            }),
            BucketNode::Map(size) => visitor.visit_map(BucketMapDeserializer {
                size: *size,
                bucket: self,
                pending: false,
            })
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        visitor.visit_some(self)
    }
}

struct BucketSeqDeserializer<'x, 'de, E> where E: serde::de::Error {
    bucket: &'x mut BucketDeserializer<'de, E>,
    size: usize,
}

impl<'x, 'de, E> SeqAccess<'de> for BucketSeqDeserializer<'x, 'de, E> where E: serde::de::Error {
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error> where T: DeserializeSeed<'de> {
        if self.size == 0 {
            Ok(None)
        } else {
            self.size -= 1;
            seed.deserialize(&mut *self.bucket)
                .map(Some)
        }
    }
}

struct BucketMapDeserializer<'x, 'de, E> where E: serde::de::Error {
    bucket: &'x mut BucketDeserializer<'de, E>,
    size: usize,
    pending: bool,
}

impl<'x, 'de, E> MapAccess<'de> for BucketMapDeserializer<'x, 'de, E> where E: serde::de::Error {
    type Error = E;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> where K: DeserializeSeed<'de> {
        if self.pending {
           return Err(E::custom("out-of-order access - no value for previous key"));
        }
        if self.size == 0 {
            return Ok(None)
        }
        self.pending = true;
        seed.deserialize(&mut *self.bucket).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error> where V: DeserializeSeed<'de> {
        if !self.pending {
            return Err(E::custom("out-of-order access - no key for this value"));
        }
        self.pending = false;
        self.size -= 1;
        seed.deserialize(&mut *self.bucket)
    }
}

pub struct BucketVisitor<'t, 'de> {
    pub(crate) target: &'t mut Vec<BucketNode<'de>>,
    pub(crate) owned: bool,
}

impl<'x, 't, 'de> Visitor<'de> for &'x mut BucketVisitor<'t, 'de> {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("valid bucket value/type")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::Bool(v));
        Ok(())
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::I8(v));
        Ok(())
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::I16(v));
        Ok(())
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::I32(v));
        Ok(())
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::I64(v));
        Ok(())
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::I128(v));
        Ok(())
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::U8(v));
        Ok(())
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::U16(v));
        Ok(())
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::U32(v));
        Ok(())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::U64(v));
        Ok(())
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::U128(v));
        Ok(())
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::F32(v));
        Ok(())
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::F64(v));
        Ok(())
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::Char(v));
        Ok(())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::String(v.to_owned()));
        Ok(())
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E> where E: Error {
        if self.owned {
            self.target.push(BucketNode::String(v.to_owned()));
        } else {
            self.target.push(BucketNode::StringRef(v));
        }
        Ok(())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::String(v));
        Ok(())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::Bytes(v.to_vec()));
        Ok(())
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E> where E: Error {
        if self.owned {
            self.target.push(BucketNode::Bytes(v.to_vec()));
        } else {
            self.target.push(BucketNode::BytesRef(v));
        }
        Ok(())
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::Bytes(v));
        Ok(())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::None);
        Ok(())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
        self.target.push(BucketNode::Some);
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> where E: Error {
        self.target.push(BucketNode::Unit);
        Ok(())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
        self.target.push(BucketNode::NewType);
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let position = self.target.len();
        let mut count = 0;
        self.target.push(BucketNode::Unsized);

        loop {
            if seq.next_element_seed(&mut *self)?.is_some() {
                count += 1;
            } else {
                break
            }
        }
        self.target[position] = BucketNode::Seq(count);
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
        let position = self.target.len();
        let mut count = 0;
        self.target.push(BucketNode::Unsized);

        while let Some(_) = map.next_key_seed(&mut *self)? {
            let _ = map.next_value_seed(&mut *self)?;
            count += 1;
        }

        self.target[position] = BucketNode::Map(count);
        Ok(())
    }
}

impl<'x, 't, 'de> DeserializeSeed<'de> for &'x mut BucketVisitor<'t, 'de> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_any(self)
    }
}