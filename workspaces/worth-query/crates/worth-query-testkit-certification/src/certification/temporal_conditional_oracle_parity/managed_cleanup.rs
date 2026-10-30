//! Real native cleanup custody survives replacement of derived conditional watches.
//! Fresh-current selection exercises cross-binding retention and reinstallation;
//! this court does not claim delivery reconsideration of the original wake.
pub use super::*;
pub use worth_query_host::facade::primary_graph::{
    BridgeExecutionBasisFinalizationFailureKind as Bridge,
    WorthQueryConditionalExecutionCause as Cause, WorthQueryManagedRunCleanupFailureKind as Native,
    WorthQueryMutationCleanupFailureKind as Failure,
};

#[path = "managed_cleanup/lag.rs"]
pub mod lag;
