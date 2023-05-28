use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter, LowerExp, Pointer, Write};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{DeserializeSeed, EnumAccess, Error, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};

pub struct Bucket<'a> {
    inner: BucketNode<'a>
}

pub struct OwnedBucket<'a> {
    inner: BucketNode<'a>
}

enum BucketNode<'a> {
    Unit,
    Bool(bool),

    U8(u8), I8(i8),
    U16(u16), I16(i16),
    U32(u32), I32(i32),
    U64(u64), I64(i64),
    U128(u128), I128(i128),
    F32(f32), F64(f64),

    Char(char),
    String(String),
    StringRef(&'a str),

    Bytes(Vec<u8>),
    BytesRef(&'a [u8]),

    None,
    Some(Box<BucketNode<'a>>),

    Seq(VecDeque<BucketNode<'a>>),
    Map(VecDeque<(BucketNode<'a>, BucketNode<'a>)>),
    Type(Box<BucketNode<'a>>),
}

impl<'a> Debug for BucketNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BucketNode::Unit => ().fmt(f),
            BucketNode::Bool(v) => v.fmt(f),
            BucketNode::U8(v) => { v.fmt(f)?; f.write_str("u8") },
            BucketNode::I8(v) => { v.fmt(f)?; f.write_str("i8") },
            BucketNode::U16(v) => { v.fmt(f)?; f.write_str("u16") },
            BucketNode::I16(v) => { v.fmt(f)?; f.write_str("i16") },
            BucketNode::U32(v) => { v.fmt(f)?; f.write_str("u32") },
            BucketNode::I32(v) => { v.fmt(f)?; f.write_str("i32") },
            BucketNode::U64(v) => { v.fmt(f)?; f.write_str("u64") },
            BucketNode::I64(v) => { v.fmt(f)?; f.write_str("i64") },
            BucketNode::U128(v) => { v.fmt(f)?; f.write_str("u128") },
            BucketNode::I128(v) => { v.fmt(f)?; f.write_str("i128") },
            BucketNode::F32(v) => { v.fmt(f)?; f.write_str("f32") },
            BucketNode::F64(v) => { v.fmt(f)?; f.write_str("f64") },
            BucketNode::Char(v) => v.fmt(f),
            BucketNode::String(v) => v.fmt(f),
            BucketNode::StringRef(v) => { f.write_str("&")?; v.fmt(f) }
            BucketNode::Bytes(v) => v.fmt(f),
            BucketNode::BytesRef(v) => { f.write_str("&")?; v.fmt(f) }
            BucketNode::None => f.debug_tuple("None").finish(),
            BucketNode::Some(v) => f.debug_tuple("Some").field(&v).finish(),
            BucketNode::Seq(seq) => {
                let mut fmt = f.debug_list();
                for val in seq { fmt.entry(&val)?; }
                fmt.finish()
            }
            BucketNode::Map(map) => {
                let mut fmt = f.debug_map();
                for (key, val) in map { fmt.entry(key, val)?; }
                fmt.finish()
            }
            BucketNode::Type(v) => f.debug_set().entry(&v).finish()
        }
    }
}


macro_rules! visit_to_typ {
    (visit_bool) => {bool};
    (visit_i8) => {i8};
    (visit_i16) => {i16};
    (visit_i32) => {i32};
    (visit_i64) => {i64};
    (visit_i128) => {i128};
    (visit_u8) => {u8};
    (visit_u16) => {u16};
    (visit_u32) => {u32};
    (visit_u64) => {u64};
    (visit_u128) => {u128};
    (visit_f32) => {f32};
    (visit_f64) => {f64};
    (visit_char) => {char};
}

macro_rules! visit {
    ($($typ: ident => $ident: ident),*) => {
        $(
        fn $typ <E>(self, v: visit_to_typ!($typ)) -> Result<Self::Value, E> where E: Error {
            Ok(Bucket::$ident(v))
        }
        )*
    };
}

impl<'de> Deserialize for BucketNode<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct BucketVisitor;

        impl<'v> Visitor<'v> for BucketVisitor {
            type Value = BucketNode<'v>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("bucket-worthy value")
            }

            visit! {
                visit_bool => Bool,
                visit_u8 => U8,
                visit_i8 => I8,
                visit_u16 => U16,
                visit_i16 => I16,
                visit_u32 => U32,
                visit_i32 => I32,
                visit_u64 => U64,
                visit_i64 => I64,
                visit_u128 => U128,
                visit_i128 => I128,
                visit_char => Char
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
                Ok(BucketNode::String(v.to_string()))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E> where E: Error {
                Ok(BucketNode::StringRef(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
                Ok(BucketNode::String(v))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: Error {
                Ok(BucketNode::Bytes(v.to_vec()))
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E> where E: Error {
                Ok(BucketNode::BytesRef(v))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> where E: Error {
                Ok(BucketNode::Bytes(v))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> where E: Error {
                Ok(BucketNode::None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
                deserializer
                    .deserialize_any(BucketVisitor)
                    .map(Box::new)
                    .map(BucketNode::Some)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> where E: Error {
                Ok(BucketNode::Unit)
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
                deserializer
                    .deserialize_any(BucketVisitor)
                    .map(Box::new)
                    .map(BucketNode::Type)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let mut vec = seq
                    .size_hint()
                    .map(VecDeque::with_capacity)
                    .unwrap_or_default();

                while let Some(element) = seq.next_element()? {
                    vec.push_back(element);
                }

                Ok(BucketNode::Seq(vec))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut vec: VecDeque<(BucketNode<'v>, BucketNode<'v>)> = map
                    .size_hint()
                    .map(VecDeque::with_capacity)
                    .unwrap_or_default();

                while let Some((key, value)) = map.next_entry()? {
                    vec.push_back((key, value));
                }

                Ok(BucketNode::Map(vec))
            }
        }

        deserializer.deserialize_any(BucketVisitor)
    }
}

impl<'a> Serialize for BucketNode<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            BucketNode::Unit => serializer.serialize_unit(),
            BucketNode::Bool(val) => serializer.serialize_bool(*val),
            BucketNode::U8(val) => serializer.serialize_u8(*val),
            BucketNode::I8(val) => serializer.serialize_i8(*val),
            BucketNode::U16(val) => serializer.serialize_u16(*val),
            BucketNode::I16(val) => serializer.serialize_i16(*val),
            BucketNode::U32(val) => serializer.serialize_u32(*val),
            BucketNode::I32(val) => serializer.serialize_i32(*val),
            BucketNode::U64(val) => serializer.serialize_u64(*val),
            BucketNode::I64(val) => serializer.serialize_i64(*val),
            BucketNode::U128(val) => serializer.serialize_u128(*val),
            BucketNode::I128(val) => serializer.serialize_i128(*val),
            BucketNode::F32(val) => serializer.serialize_f32(*val),
            BucketNode::F64(val) => serializer.serialize_f64(*val),
            BucketNode::Char(val) => serializer.serialize_char(*val),
            BucketNode::String(val) => serializer.serialize_str(&val),
            BucketNode::StringRef(val) => serializer.serialize_str(val),
            BucketNode::Bytes(val) => serializer.serialize_bytes(val),
            BucketNode::BytesRef(val) => serializer.serialize_bytes(val),
            BucketNode::None => serializer.serialize_none(),
            BucketNode::Some(val) => serializer.serialize_some(val.as_ref()),
            BucketNode::Seq(val) => {
                let mut ser = serializer.serialize_seq(Some(val.len()))?;
                for x in val {
                    ser.serialize_element(&x)
                }
                ser.end()
            }
            BucketNode::Map(val) => {
                let mut ser = serializer.serialize_map(Some(val.len()))?;
                for (k, v) in val {
                    ser.serialize_entry(&k, &v)?;
                }
                ser.end()
            }
            BucketNode::Type(typ) => serializer.serialize_newtype_struct("", typ)
        }
    }
}

impl<'a> Deserializer<'a> for BucketNode<'a> {
    type Error = ();

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {

    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_struct<V>(self, name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_enum<V>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'a> {
        todo!()
    }

    fn is_human_readable(&self) -> bool {
        todo!()
    }
}