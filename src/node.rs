
/// the inner type of a node queue
#[derive(Default)]
pub(crate) enum BucketNode<'a> {
    #[default]
    Consumed,

    Unsized,

    Unit,
    Bool(bool),
    Char(char),

    U8(u8), I8(i8),
    U16(u16), I16(i16),
    U32(u32), I32(i32),
    U64(u64), I64(i64),
    U128(u128), I128(i128),
    F32(f32), F64(f64),

    /*#[cfg(feature = "alloc")]*/
    String(String),
    StringRef(&'a str),

    /*#[cfg(feature = "alloc")]*/
    Bytes(Vec<u8>),
    BytesRef(&'a [u8]),

    None,
    Some,

    Seq(usize),
    Map(usize),

    NewType,
}
