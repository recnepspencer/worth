//! Committed change, exact reads, lineage, commit selection, and snapshot
//! retention at a retained observation, in Relational vocabulary only.
//!
//! A consumer that projects Relational truth into its own model reads it
//! through this module. Nothing here names a consumer concept.

pub(crate) use worth_relational_branch::internal_port::change_source::commit_selection;
pub(crate) use worth_relational_contracts::internal_port::change_source::consistency;
#[cfg(test)]
mod consistency_tests;
mod exact_reads;
mod lineage_at_observation;
pub(crate) use worth_relational_contracts::internal_port::change_source::memory_close_observation;
mod names_at_observation;
mod names_at_snapshot;
mod observation_read;
mod partition_projection;
mod pinned_commit;
pub use pinned_commit::{RelationalPinnedCommitDenial, RelationalPinnedCommitObservation};
pub(crate) use worth_relational_branch::internal_port::change_source::receipt;
pub(crate) use worth_relational_contracts::internal_port::change_source::read_custody;
mod receipt_minting;
pub(crate) use worth_relational_branch::internal_port::change_source::receipt_witness;
mod retain_observation;
pub(crate) use worth_relational_branch::internal_port::change_source::retained_observation;
mod runtime_handle;
mod select_commit;
pub use read_custody::RelationalReadCustodyHolders;

pub use commit_selection::{
    RelationalCommitSelection, RelationalCommitSelectionDenial, RelationalCommitSelectionWork,
    RelationalSelectedCommit,
};
pub use consistency::{
    RelationalChangeConsistencyDenial, RelationalChangeConsistencyDenialKind,
    RelationalChangeConsistencyWork,
};
pub use names_at_observation::RelationalPersistentNameReadDenial;
pub use observation_read::RelationalObservationReadDenial;
pub use receipt::{
    RelationalChangeReceipt, RelationalChangeReceiptDeferred, RelationalChangeReceiptOutcome,
    RelationalChangeReceiptStale,
};
pub use retained_observation::{
    RelationalRetainedObservation, RelationalRetainedObservationRetirement,
    RelationalRetainedObservationRetirementDenial, RelationalRetainedObservationRetirementProgress,
    RetiredRelationalRetainedObservation,
};
pub use runtime_handle::{
    RelationalMemoryClose, RelationalMemoryCloseDenial, RelationalMemoryCloseObservation,
    RelationalMemoryCloseRefused, RetiredMemoryRelationalAllocation,
};
pub use runtime_handle::{
    RelationalRuntimeHandle, RelationalRuntimeHandleAliasRetirementDenial,
    RelationalRuntimeLockObservationForTest, RetiredRelationalRuntimeHandleAlias,
};
