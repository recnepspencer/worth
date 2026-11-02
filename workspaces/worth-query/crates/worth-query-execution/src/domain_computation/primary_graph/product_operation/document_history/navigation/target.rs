//! Descriptive navigation intent; only the admitted journal can resolve an edge.
use super::super::WorthQueryHistoryNodeId;

/// Describes a target without granting model, metadata or navigation authority.
/// Parent and child refer to the original expected user step, not a later head.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryNavigationIntent {
    Checkout(WorthQueryHistoryNodeId),
    UndoStep,
    RedoTo(WorthQueryHistoryNodeId),
}
