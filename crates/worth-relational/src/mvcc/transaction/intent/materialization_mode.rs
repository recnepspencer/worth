//! Select the authored materialization transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalMaterializationTransactionMode {
    Suspend,
    Rematerialize,
}
