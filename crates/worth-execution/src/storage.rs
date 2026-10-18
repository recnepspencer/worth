//! Allocation custody for retained runtime data, separate from execution results.
pub(crate) mod ordered;
pub use ordered::{ExecutionOrderedMap, ExecutionOrderedMapDenial};
