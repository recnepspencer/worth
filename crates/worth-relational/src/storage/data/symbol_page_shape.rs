//! Scalar shape of one authenticated dictionary page, without source custody.
use crate::symbols::data::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SymbolPageShape {
    pub(crate) first: Symbol,
    pub(crate) last: Symbol,
    pub(crate) homogeneous_bucket: Option<u8>,
    pub(crate) id_digest: [u8; 32],
}
