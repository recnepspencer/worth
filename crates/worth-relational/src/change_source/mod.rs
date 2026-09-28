//! Committed change, exact reads, lineage, commit selection, and snapshot
//! retention at a retained observation, in Relational vocabulary only.
//!
//! A consumer that projects Relational truth into its own model reads it
//! through this module. Nothing here names a consumer concept.

mod commit_selection;
mod consistency;
mod consistency_expectations;
#[cfg(test)]
mod consistency_tests;
mod exact_reads;
mod lineage_at_observation;
mod partition_projection;
mod receipt;
mod receipt_minting;
mod receipt_witness;
mod retained_observation;
mod runtime_handle;

pub use commit_selection::{
    RelationalCommitSelection, RelationalCommitSelectionDenial, RelationalCommitSelectionWork,
    RelationalSelectedCommit,
};
pub use consistency::{
    RelationalChangeConsistencyDenial, RelationalChangeConsistencyDenialKind,
    RelationalChangeConsistencyWork,
};
pub use receipt::{
    RelationalChangeReceipt, RelationalChangeReceiptDeferred, RelationalChangeReceiptOutcome,
    RelationalChangeReceiptStale,
};
pub use retained_observation::RelationalRetainedObservation;
pub use runtime_handle::RelationalRuntimeHandle;
