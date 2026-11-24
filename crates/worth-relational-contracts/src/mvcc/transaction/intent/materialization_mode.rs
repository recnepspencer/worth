//! Select the authored materialization transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalMaterializationTransactionMode {
    Suspend,
    Rematerialize,
}
