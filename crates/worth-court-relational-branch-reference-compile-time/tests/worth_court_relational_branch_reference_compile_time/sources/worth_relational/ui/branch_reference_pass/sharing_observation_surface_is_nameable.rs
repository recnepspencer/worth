//! The sharing-observation surface is reachable and exactly shaped through the
//! public facade alone.
//!
//! The observation and its vocabulary live in several private sibling modules.
//! This court binds every public accessor to an explicit function-pointer type
//! through `worth_relational::facade::inspection`, so a moved declaration, a
//! renamed accessor, a changed return type, a dropped derive, or a lost
//! re-export fails to compile here rather than silently reshaping the surface.

use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::inspection::{
    RelationalAuthoritativeAllocationKind, RelationalAuthoritativeAllocationLocator,
    RelationalAuthoritativeAllocationObservation, RelationalBranchSharingInspectionDenial,
    RelationalBranchSharingObservation, RelationalCorrectnessIndexPosture,
    RelationalSharingByteMetricScope, RelationalStorageRegionLocator,
    RelationalVisibilityCommitmentObservation, RELATIONAL_SHARING_INSPECTION_VERSION,
};

type Byte = fn(&RelationalBranchSharingObservation) -> u64;

const DECLARED_VERSION: u16 = RELATIONAL_SHARING_INSPECTION_VERSION;

fn require_evidence_derives<T: std::fmt::Debug + Clone + PartialEq + Eq>() {}

fn require_copy_evidence<T: std::fmt::Debug + Copy + PartialEq + Eq>() {}

fn declaration_and_selection_lanes() {
    let _: fn(&RelationalBranchSharingObservation) -> u16 =
        RelationalBranchSharingObservation::inspection_version;
    let _: fn(&RelationalBranchSharingObservation) -> RelationalSharingByteMetricScope =
        RelationalBranchSharingObservation::byte_metric_scope;
    let _: Byte = RelationalBranchSharingObservation::branch_count;
    let _: Byte = RelationalBranchSharingObservation::unique_root_count;
    let _: Byte = RelationalBranchSharingObservation::unique_canonical_commit_artifacts;
    let _: Byte = RelationalBranchSharingObservation::branch_metadata_bytes;
    let _: fn(&RelationalBranchSharingObservation) -> &[u64] =
        RelationalBranchSharingObservation::root_ids;
}

fn authoritative_byte_lane() {
    let _: Byte = RelationalBranchSharingObservation::logical_branch_partition_payload_bytes;
    let _: Byte = RelationalBranchSharingObservation::unique_physical_partition_payload_bytes;
    let _: Byte = RelationalBranchSharingObservation::logical_branch_root_metadata_bytes;
    let _: Byte = RelationalBranchSharingObservation::unique_physical_root_metadata_bytes;
    let _: Byte = RelationalBranchSharingObservation::logical_branch_root_reachability_bytes;
    let _: Byte = RelationalBranchSharingObservation::unique_physical_root_reachability_bytes;
    let _: Byte = RelationalBranchSharingObservation::logical_branch_canonical_commit_bytes;
    let _: Byte = RelationalBranchSharingObservation::unique_physical_canonical_commit_bytes;
    let _: Byte = RelationalBranchSharingObservation::logical_branch_authoritative_bytes;
    let _: Byte = RelationalBranchSharingObservation::unique_physical_authoritative_bytes;
    let _: Byte = RelationalBranchSharingObservation::unique_diagnostic_bytes;
    let _: Byte = RelationalBranchSharingObservation::unique_retention_metadata_bytes;
    let _: Byte = RelationalBranchSharingObservation::unique_allocator_bookkeeping_bytes;
    let _: Byte = RelationalBranchSharingObservation::unique_optional_cache_bytes;
}

fn authoritative_evidence_lane() {
    let _: Byte = RelationalBranchSharingObservation::inspection_reconstructed_region_count;
    let _: fn(&RelationalBranchSharingObservation) -> RelationalCorrectnessIndexPosture =
        RelationalBranchSharingObservation::correctness_index_posture;
    let _: fn(&RelationalBranchSharingObservation) -> &[RelationalStorageRegionLocator] =
        RelationalBranchSharingObservation::region_locators;
    let _: fn(
        &RelationalBranchSharingObservation,
    ) -> &[RelationalAuthoritativeAllocationObservation] =
        RelationalBranchSharingObservation::authoritative_allocations;
    let _: fn(&RelationalBranchSharingObservation) -> &[RelationalVisibilityCommitmentObservation] =
        RelationalBranchSharingObservation::visibility_commitments;
}

fn recorded_cost_lane() {
    let _: Byte = RelationalBranchSharingObservation::copied_truth_bytes;
    let _: Byte = RelationalBranchSharingObservation::copied_commit_envelopes;
    let _: Byte = RelationalBranchSharingObservation::fork_materialized_entity_count;
    let _: Byte = RelationalBranchSharingObservation::fork_materialized_relation_count;
    let _: Byte = RelationalBranchSharingObservation::fork_materialized_authoritative_bytes;
    let _: Byte = RelationalBranchSharingObservation::shared_root_acquisitions;
    let _: Byte = RelationalBranchSharingObservation::publication_touched_region_count;
    let _: Byte = RelationalBranchSharingObservation::publication_reused_region_count;
    let _: Byte = RelationalBranchSharingObservation::publication_new_authoritative_bytes;
    let _: Byte = RelationalBranchSharingObservation::reclaimable_unique_bytes;
}

fn coordination_lane() {
    let _: Byte = RelationalBranchSharingObservation::coordination_contacts;
    let _: Byte = RelationalBranchSharingObservation::coordination_waits;
    // `RelationalBranchCoordinationCellId` has no public facade path, so this
    // accessor is frozen by use rather than by naming its element type.
    fn cell_count(observation: &RelationalBranchSharingObservation) -> usize {
        observation.coordination_cells().len()
    }
    let _: fn(&RelationalBranchSharingObservation) -> usize = cell_count;
}

fn allocation_vocabulary() {
    let _: fn(RelationalAuthoritativeAllocationLocator) -> u64 =
        RelationalAuthoritativeAllocationLocator::runtime_instance_id;
    let _: fn(RelationalAuthoritativeAllocationLocator) -> RelationalAuthoritativeAllocationKind =
        RelationalAuthoritativeAllocationLocator::kind;
    let _: fn(RelationalAuthoritativeAllocationLocator) -> u64 =
        RelationalAuthoritativeAllocationLocator::owner_id;
    let _: fn(RelationalAuthoritativeAllocationLocator) -> u64 =
        RelationalAuthoritativeAllocationLocator::creation_owner_id;
    let _: fn(RelationalAuthoritativeAllocationLocator) -> Option<PartitionId> =
        RelationalAuthoritativeAllocationLocator::partition_id;
    let _: fn(
        RelationalAuthoritativeAllocationObservation,
    ) -> RelationalAuthoritativeAllocationLocator =
        RelationalAuthoritativeAllocationObservation::locator;
    let _: fn(RelationalAuthoritativeAllocationObservation) -> u64 =
        RelationalAuthoritativeAllocationObservation::authoritative_bytes;
    let _: fn(RelationalStorageRegionLocator) -> u64 =
        RelationalStorageRegionLocator::runtime_instance_id;
    let _: fn(RelationalStorageRegionLocator) -> u64 = RelationalStorageRegionLocator::root_id;
    let _: fn(RelationalStorageRegionLocator) -> u64 = RelationalStorageRegionLocator::region_id;
    let _: fn(RelationalStorageRegionLocator) -> PartitionId =
        RelationalStorageRegionLocator::partition_id;
    let _: fn(RelationalVisibilityCommitmentObservation) -> u64 =
        RelationalVisibilityCommitmentObservation::root_id;
    let _: fn(RelationalVisibilityCommitmentObservation) -> [u8; 32] =
        RelationalVisibilityCommitmentObservation::digest;
    let _ = RelationalAuthoritativeAllocationKind::PartitionPayload;
    let _ = RelationalCorrectnessIndexPosture::AuthoritativeFallback;
    let _ = RelationalBranchSharingInspectionDenial::ForeignRuntime;
}

fn declared_scopes() {
    let _ = RelationalSharingByteMetricScope::CompleteAuthoritativeOwnerAllocations;
    // The historical scope stays nameable so that existing matches keep
    // compiling; version 4 never produces it.
    #[allow(deprecated)]
    let _ = RelationalSharingByteMetricScope::AuthoritativePartitionPayloadsOnly;
}

fn main() {
    assert_eq!(DECLARED_VERSION, RELATIONAL_SHARING_INSPECTION_VERSION);
    require_evidence_derives::<RelationalBranchSharingObservation>();
    require_evidence_derives::<RelationalBranchSharingInspectionDenial>();
    require_copy_evidence::<RelationalSharingByteMetricScope>();
    require_copy_evidence::<RelationalAuthoritativeAllocationKind>();
    require_copy_evidence::<RelationalAuthoritativeAllocationLocator>();
    require_copy_evidence::<RelationalAuthoritativeAllocationObservation>();
    require_copy_evidence::<RelationalStorageRegionLocator>();
    require_copy_evidence::<RelationalVisibilityCommitmentObservation>();
    require_copy_evidence::<RelationalCorrectnessIndexPosture>();
    declaration_and_selection_lanes();
    authoritative_byte_lane();
    authoritative_evidence_lane();
    recorded_cost_lane();
    coordination_lane();
    allocation_vocabulary();
    declared_scopes();
}
