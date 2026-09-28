//! The versioned draft codec: an untrusted artifact boundary.
//!
//! Drafts encode as syntax without source spans. Decoding checks versions,
//! then every count against the profile and the remaining input before it
//! allocates, and re-establishes the tree invariants admission relies on:
//! post-order children with one parent each, a single root, the node and
//! depth ceilings, identifiers, and normalized decimal text.
//!
//! Layout, little-endian: magic `WXDR`, wire version `u16`, language version
//! `u16`, node count `u32`, then nodes in post-order. Each node is a tag byte
//! and its payload; children are `u32` indices of earlier nodes, each used
//! exactly once, and the last node is the root.

mod decode;
mod encode;

pub(crate) use decode::decode_draft;
pub(crate) use encode::encode_draft;

const MAGIC: [u8; 4] = *b"WXDR";
const WIRE_VERSION: u16 = 1;
const LANGUAGE_VERSION: u16 = 1;

mod tag {
    pub(super) const BOOL: u8 = 0;
    pub(super) const INTEGER: u8 = 1;
    pub(super) const FLOAT: u8 = 2;
    pub(super) const STRING: u8 = 3;
    pub(super) const NAME: u8 = 4;
    pub(super) const FIELD: u8 = 5;
    pub(super) const COMPREHENSION: u8 = 6;
    pub(super) const UNARY: u8 = 7;
    pub(super) const BINARY: u8 = 8;
    pub(super) const CONDITIONAL: u8 = 9;
    pub(super) const LET: u8 = 10;
    pub(super) const CALL: u8 = 11;
    pub(super) const NONE: u8 = 12;
    pub(super) const LIST: u8 = 13;
    pub(super) const MAP: u8 = 14;
    pub(super) const RECORD: u8 = 15;

    pub(super) const TYPE: u8 = 0;
    pub(super) const WIDTH: u8 = 1;
    pub(super) const PRODUCT: u8 = 2;
}

use super::syntax::ast::{BinaryOp, ComprehensionKind, UnaryOp};

const UNARY_OPS: [UnaryOp; 2] = [UnaryOp::Not, UnaryOp::Negate];
const BINARY_OPS: [BinaryOp; 14] = [
    BinaryOp::Multiply,
    BinaryOp::Divide,
    BinaryOp::Remainder,
    BinaryOp::Add,
    BinaryOp::Subtract,
    BinaryOp::Less,
    BinaryOp::LessEqual,
    BinaryOp::Greater,
    BinaryOp::GreaterEqual,
    BinaryOp::Equal,
    BinaryOp::NotEqual,
    BinaryOp::And,
    BinaryOp::Or,
    BinaryOp::Coalesce,
];
const COMPREHENSIONS: [ComprehensionKind; 4] = [
    ComprehensionKind::Map,
    ComprehensionKind::Filter,
    ComprehensionKind::All,
    ComprehensionKind::Any,
];

/// The wire code of `value` in its closed operator table.
fn code<T: PartialEq>(table: &[T], value: &T) -> u8 {
    table
        .iter()
        .position(|known| known == value)
        .expect("operator tables are complete") as u8
}
