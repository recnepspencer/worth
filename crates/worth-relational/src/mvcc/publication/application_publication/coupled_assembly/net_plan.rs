//! Incremental C-to-final assembly keeps allocation custody separate from effects.
pub(super) mod creator_custody;
pub(super) use creator_custody::CreatorCustody;
