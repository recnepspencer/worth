//! Committed change and exact reads at a retained observation, for a
//! consumer that projects Relational truth into its own model.
//!
//! The entry points are methods on types a consumer already holds:
//!
//! 1. Hold the runtime: [`RelationalRuntimeHandle`] covers an immutable or a
//!    mutex-shared runtime.
//! 2. Retain what you read at:
//!    [`RelationalRuntime::retain_observation_snapshot`](crate::facade::runtime::RelationalRuntime::retain_observation_snapshot)
//!    turns an admitted branch basis into a [`RelationalRetainedObservation`]
//!    with its own snapshot id.
//! 3. Select a commit the observation can see:
//!    [`RelationalRuntime::select_exact_commit`](crate::facade::runtime::RelationalRuntime::select_exact_commit)
//!    at constant cost, or
//!    [`RelationalRuntime::select_reachable_commit`](crate::facade::runtime::RelationalRuntime::select_reachable_commit)
//!    through one ancestry walk. Each returns a [`RelationalCommitSelection`]
//!    carrying its work.
//! 4. Mint the change:
//!    [`RelationalRuntime::mint_change_receipt`](crate::facade::runtime::RelationalRuntime::mint_change_receipt)
//!    consumes a [`RelationalSelectedCommit`] and returns a
//!    [`RelationalChangeReceipt`] whose patch passed the consistency rules.
//!    A patch held by any other route is checked with
//!    [`PublishedAuthoritativePatchEnvelope::check_change_consistency`](crate::facade::publication::PublishedAuthoritativePatchEnvelope::check_change_consistency).
//! 5. Read at the observation:
//!    [`entity_record_at_observation`](crate::facade::runtime::RelationalRuntime::entity_record_at_observation),
//!    [`relation_record_at_observation`](crate::facade::runtime::RelationalRuntime::relation_record_at_observation),
//!    [`record_history_at_observation`](crate::facade::runtime::RelationalRuntime::record_history_at_observation),
//!    and
//!    [`visible_entities_for_lineages_at_observation`](crate::facade::runtime::RelationalRuntime::visible_entities_for_lineages_at_observation)
//!    refuse an observation another runtime issued with
//!    [`RelationalObservationReadDenial`]. The retained schema answers
//!    [`entity_kind_declares_aspect`](crate::facade::mvcc::RelationalBranchObservation::entity_kind_declares_aspect)
//!    and
//!    [`relation_kind_declares_aspect`](crate::facade::mvcc::RelationalBranchObservation::relation_kind_declares_aspect).

pub use crate::change_source::{
    RelationalChangeConsistencyDenial, RelationalChangeConsistencyDenialKind,
    RelationalChangeConsistencyWork, RelationalChangeReceipt, RelationalChangeReceiptDeferred,
    RelationalChangeReceiptOutcome, RelationalChangeReceiptStale, RelationalCommitSelection,
    RelationalCommitSelectionDenial, RelationalCommitSelectionWork,
    RelationalObservationReadDenial, RelationalRetainedObservation, RelationalRuntimeHandle,
    RelationalSelectedCommit,
};
