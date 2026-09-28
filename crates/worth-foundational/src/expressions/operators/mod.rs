//! Value operators for the evaluator: each computes one operation exactly and
//! reports the semantic work it performed, so evaluation can charge it.

pub(crate) mod conversion;
pub(crate) mod decimal;
pub(crate) mod digital;
pub(crate) mod float;
pub(crate) mod integer;
pub(crate) mod quantity;
pub(crate) mod text;
