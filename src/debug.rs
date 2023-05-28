use std::fmt;
use std::fmt::{Debug, Formatter, Write};
use serde::ser::Error;
use crate::node::BucketNode;

pub(crate) fn debug_nodes(nodes: &[BucketNode], fmt: &mut Formatter) -> fmt::Result {
    let mut cursor = 0;
    while nodes.len() > cursor {
        debug_node(&nodes, fmt, &mut cursor)?;
    }
    Ok(())
}

pub(crate) fn debug_node(nodes: &[BucketNode], fmt: &mut Formatter, cursor: &mut usize) -> fmt::Result {
    let Some(node) = nodes.get(*cursor) else {
        return Err(fmt::Error::custom("out-of-bounds"))
    };
    match node {
        BucketNode::Consumed | BucketNode::Unsized => fmt.write_char('_'),

        BucketNode::Unit => ().fmt(fmt),
        BucketNode::Bool(val) => val.fmt(fmt),
        BucketNode::Char(val) => val.fmt(fmt),

        BucketNode::U8(val) => { val.fmt(fmt)?; fmt.write_str("u8") },
        BucketNode::I8(val) => { val.fmt(fmt)?; fmt.write_str("i8") },
        BucketNode::U16(val) => { val.fmt(fmt)?; fmt.write_str("u16") },
        BucketNode::I16(val) => { val.fmt(fmt)?; fmt.write_str("i16") },
        BucketNode::U32(val) => { val.fmt(fmt)?; fmt.write_str("u32") },
        BucketNode::I32(val) => { val.fmt(fmt)?; fmt.write_str("i32") },
        BucketNode::U64(val) => { val.fmt(fmt)?; fmt.write_str("u64") },
        BucketNode::I64(val) => { val.fmt(fmt)?; fmt.write_str("i64") },
        BucketNode::U128(val) => { val.fmt(fmt)?; fmt.write_str("u128") },
        BucketNode::I128(val) => { val.fmt(fmt)?; fmt.write_str("i128") },
        BucketNode::F32(val) => { val.fmt(fmt)?; fmt.write_str("f32") },
        BucketNode::F64(val) => { val.fmt(fmt)?; fmt.write_str("f64") },

        BucketNode::String(val) => val.fmt(fmt),
        BucketNode::StringRef(val) => { fmt.write_str("&")?; val.fmt(fmt) }

        BucketNode::Bytes(val) => val.fmt(fmt),
        BucketNode::BytesRef(val) => { fmt.write_str("&")?; val.fmt(fmt) }

        BucketNode::None => fmt.write_str("None"),
        BucketNode::Some => {
            *cursor += 1;
            fmt.write_str("Some(")?;
            debug_node(&nodes, fmt, cursor)?;
            return fmt.write_str(")")
        }
        BucketNode::Seq(seq) => {
            *cursor += 1;
            fmt.write_str("[")?;

            for i in 0..*seq {
                if i != 0 { fmt.write_str(", ")?; }
                debug_node(&nodes, fmt, cursor)?;
            }

            return fmt.write_str("]")
        }
        BucketNode::Map(map) => {
            *cursor += 1;
            fmt.write_str("{")?;

            for i in 0..*map {
                if i != 0 { fmt.write_str(", ")?; }
                debug_node(&nodes, fmt, cursor)?;
                fmt.write_str(": ")?;
                debug_node(&nodes, fmt, cursor)?;
            }

            return fmt.write_str("}")
        }
        BucketNode::NewType => {
            *cursor += 1;
            fmt.write_str("(")?;
            debug_node(&nodes, fmt, cursor)?;
            return fmt.write_str(")")
        }

    }?;
    *cursor += 1;
    Ok(())
}
