//! Value operators for the evaluator: each computes one operation exactly.
//! Their work is charged by the evaluator, statically in `evaluation::cost`
//! or as contents are visited, not reported by the operators.

pub(crate) mod conversion;
pub(crate) mod decimal;
pub(crate) mod digital;
pub(crate) mod float;
pub(crate) mod integer;
pub(crate) mod quantity;
pub(crate) mod text;
