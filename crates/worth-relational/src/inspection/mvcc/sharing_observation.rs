use crate::branch::RelationalBranchCoordinationCellId;

use super::allocation_inventory::{
    RelationalAuthoritativeAllocationObservation, RelationalStorageRegionLocator,
};
use super::byte_metric_scope::RelationalSharingByteMetricScope;
use super::root_commitment::{
    RelationalCorrectnessIndexPosture, RelationalVisibilityCommitmentObservation,
};

/// Read-only evidence about how a chosen set of branches shares owner storage.
///
/// # Truth-source lanes
///
/// Every metric on this type belongs to exactly one lane, and the lanes are
/// read from different places at different times. Metrics from different lanes
/// are never substitutable for one another, even when their units agree:
///
/// | Lane | Read from | When |
/// | --- | --- | --- |
/// | Declaration | compile-time constants of the inspection module | assembly |
/// | Selection | the caller's own branch slice | assembly |
/// | Live authoritative bytes | the selected roots, regions, and commit artifacts | observation time |
/// | Live authoritative evidence | the same owner walk, kept as locators and commitments | observation time |
/// | Recorded cost | per-branch counters written by earlier fork and publication work | when that work ran |
/// | Coordination | the selected branches' coordination cells | observation time |
///
/// The live lanes describe storage that exists right now; the recorded-cost
/// lane describes work that already happened and is never recomputed from live
/// state. A branch may therefore report large live totals with zero recorded
/// costs (it shares a root it never published to) or nonzero recorded costs
/// with small live totals (its published storage was superseded).
///
/// # Byte scope
///
/// The live authoritative byte totals are governed by
/// [`RelationalSharingByteMetricScope`], which every observation states
/// explicitly through [`Self::byte_metric_scope`]. That scope does not govern
/// [`Self::branch_metadata_bytes`], which is a selection-lane metric with its
/// own narrower scope, and it does not govern any recorded cost counter.
/// No metric on this type is a total resident-memory measurement.
///
/// # Authority
///
/// This is evidence only. No accessor yields a branch basis, a transaction
/// binding, or any other capability, and no locator here can be turned back
/// into a root, a branch, or a region handle.
///
/// ```
/// use worth_relational::facade::inspection::{
///     RelationalSharingByteMetricScope, RELATIONAL_SHARING_INSPECTION_VERSION,
/// };
/// use worth_relational::facade::mvcc::{
///     RelationalPublicationOutcome, RelationalTransactionIntent,
/// };
/// use worth_relational::facade::runtime::RelationalRuntimeApi;
/// use worth_relational::facade::schema::RelationalSchemaRegistry;
/// use worth_relational::facade::transactions::WorkerIntentBatch;
///
/// let mut runtime = RelationalRuntimeApi::builder()
///     .schema_registry(RelationalSchemaRegistry::new())
///     .build();
///
/// // Sharing evidence is reported over complete, artifact-linked roots only,
/// // so the main branch publishes once before it can be observed.
/// let identity = runtime.main_branch_identity();
/// let (_descriptor, basis) = runtime
///     .observe_branch(&identity)
///     .expect("the main branch is observable");
/// let mut transaction = runtime
///     .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
///     .expect("the exact basis admits a branch transaction");
/// transaction
///     .push_batch(WorkerIntentBatch::new("sharing-observation-doc"), worth_execution::ExecutionAllocationPolicy::SystemAllocation)
///     .expect("an empty batch stages");
/// let candidate = runtime
///     .prepare_branch_transaction(transaction, worth_execution::ExecutionAllocationPolicy::SystemAllocation,)
///     .expect("preparation validates");
/// let performed = match runtime.publication_port().compare_and_publish(candidate) {
///     RelationalPublicationOutcome::Performed(performed) => performed,
///     outcome => panic!("unexpected publication outcome: {outcome:?}"),
/// };
/// let committed = runtime
///     .settle_performed_publication(performed)
///     .expect("the performed publication settles through its owner");
/// runtime
///     .snapshots()
///     .release_snapshot(&committed.snapshot)
///     .expect("the commit snapshot releases exactly once");
///
/// let observation = runtime
///     .observe_branch_sharing(&[runtime.main_branch_identity()])
///     .expect("a published main branch is inspectable");
///
/// // Declaration lane: the contract the remaining metrics are read under.
/// assert_eq!(
///     observation.inspection_version(),
///     RELATIONAL_SHARING_INSPECTION_VERSION
/// );
/// assert_eq!(
///     observation.byte_metric_scope(),
///     RelationalSharingByteMetricScope::CompleteAuthoritativeOwnerAllocations
/// );
///
/// // Selection lane: the caller's own selection, not a live owner reading.
/// assert_eq!(observation.branch_count(), 1);
/// assert_eq!(observation.unique_root_count(), 1);
///
/// // Live authoritative lanes: the byte total is exactly the deduplicated
/// // allocation evidence, so callers can re-derive it themselves.
/// let summed: u64 = observation
///     .authoritative_allocations()
///     .iter()
///     .map(|allocation| allocation.authoritative_bytes())
///     .sum();
/// assert_eq!(summed, observation.unique_physical_authoritative_bytes());
///
/// // Recorded-cost lane: the main branch was never forked, whatever storage
/// // it holds now.
/// assert_eq!(observation.shared_root_acquisitions(), 0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalBranchSharingObservation {
    pub(super) inspection_version: u16,
    pub(super) byte_metric_scope: RelationalSharingByteMetricScope,
    pub(super) branch_count: u64,
    pub(super) unique_root_count: u64,
    pub(super) root_ids: Vec<u64>,
    pub(super) unique_canonical_commit_artifacts: u64,
    pub(super) logical_branch_partition_payload_bytes: u64,
    pub(super) unique_physical_partition_payload_bytes: u64,
    pub(super) logical_branch_root_metadata_bytes: u64,
    pub(super) unique_physical_root_metadata_bytes: u64,
    pub(super) logical_branch_root_reachability_bytes: u64,
    pub(super) unique_physical_root_reachability_bytes: u64,
    pub(super) logical_branch_canonical_commit_bytes: u64,
    pub(super) unique_physical_canonical_commit_bytes: u64,
    pub(super) logical_branch_authoritative_bytes: u64,
    pub(super) unique_physical_authoritative_bytes: u64,
    pub(super) unique_diagnostic_bytes: u64,
    pub(super) unique_retention_metadata_bytes: u64,
    pub(super) unique_allocator_bookkeeping_bytes: u64,
    pub(super) unique_optional_cache_bytes: u64,
    pub(super) branch_metadata_bytes: u64,
    pub(super) copied_truth_bytes: u64,
    pub(super) copied_commit_envelopes: u64,
    pub(super) fork_materialized_entity_count: u64,
    pub(super) fork_materialized_relation_count: u64,
    pub(super) fork_materialized_authoritative_bytes: u64,
    pub(super) shared_root_acquisitions: u64,
    pub(super) publication_touched_region_count: u64,
    pub(super) publication_reused_region_count: u64,
    pub(super) publication_new_authoritative_bytes: u64,
    pub(super) reclaimable_unique_bytes: u64,
    pub(super) coordination_contacts: u64,
    pub(super) coordination_waits: u64,
    pub(super) correctness_index_posture: RelationalCorrectnessIndexPosture,
    pub(super) coordination_cells: Vec<RelationalBranchCoordinationCellId>,
    pub(super) region_locators: Vec<RelationalStorageRegionLocator>,
    pub(super) authoritative_allocations: Vec<RelationalAuthoritativeAllocationObservation>,
    pub(super) visibility_commitments: Vec<RelationalVisibilityCommitmentObservation>,
    pub(super) inspection_reconstructed_region_count: u64,
}
