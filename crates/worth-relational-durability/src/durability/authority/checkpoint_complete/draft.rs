//! Assemble the same concrete manifest DATA without materializing legacy images.
use super::canonical::CanonicalOutput;
use super::metadata::CheckpointMetadataMembers;
use crate::durability::data::*;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
pub(super) struct CheckpointDraftMetadata<'a> {
    pub(super) branches: &'a worth_relational_branch::internal_port::branch::registry::checkpoint_capture::data::CheckpointBranchMetadata,
    pub(super) identity: &'a worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::paged_base::FrozenRecordIdentity,
    /// The destination's current members: pages it holds may be carried.
    pub(super) destination: &'a [RelationalDurableMemberLocator],
    pub(super) indexes: &'a worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::checkpoint_inputs::OriginalCheckpointIndexes,
    pub(super) symbol_frontier: u32,
    pub(super) through: contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
}
pub(super) fn assemble(
    source: &CheckpointDraftMetadata<'_>,
    canonical: &CanonicalOutput,
    schemas: CheckpointSchemaImages,
    lineage_basis: CheckpointLineageBasis,
    metadata: CheckpointMetadataMembers,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    (
        RelationalCheckpointManifestDraft,
        ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    ),
    Denial,
> {
    let last = canonical.rows.last().map(|row| &row.commit);
    let bytes =
        match last {
            None => 0,
            Some(receipt) => std::alloc::Layout::array::<
                contracts::history::data::coordinates::CommitId,
            >(receipt.parents.len())
            .ok()
            .and_then(|parents| parents.size().checked_add(receipt.branch_id.0.len()))
            .ok_or_else(overflow)?,
        };
    let memory = if bytes == 0 {
        None
    } else {
        request
            .reserve_retained_memory(u64::try_from(bytes).map_err(|_| overflow())?)
            .map_err(|cause| Denial::Request(cause.into()))?
    };
    let coverage = CheckpointCoverage {
        up_to_commit: last.cloned(),
        up_to_version: last.map(|commit| commit.version_id),
        up_to_branch_transition: source.through,
    };
    let definitions =
        CheckpointIndexDefinitions::capture_for_request(&source.indexes.catalog, request)
            .map_err(Denial::Request)?;
    let mut members = metadata.members;
    let record_identity = super::identity_members::capture(
        source.identity,
        source.destination,
        &mut members,
        request,
    )?;
    let draft = RelationalCheckpointManifestDraft {
        branch_data: source.branches.clone(),
        coverage,
        covered_position: canonical.rows.last().map_or(
            contracts::publication::patch::data::patch_position::PatchStreamPosition(0),
            |row| row.position,
        ),
        schemas,
        record_identity,
        aspect_contracts: Vec::new(),
        index_generation_frontier: source.indexes.generation_frontier,
        index_definition_frontier: source.indexes.catalog.next_index_id,
        index_definitions: definitions,
        roots: Default::default(),
        retained_empty_components: Vec::new(),
        mirror_schema: None,
        mirror: Vec::new(),
        history_dependencies: Vec::new(),
        // Paged once the regions install the selected roots.
        commits: CheckpointCommitIndex::default(),
        canonical_members: canonical.inventory.clone(),
        symbols: metadata.symbols,
        symbol_identity_frontier: source.symbol_frontier,
        symbol_buckets: metadata.symbol_buckets,
        lineage: metadata.lineage,
        lineage_events: metadata.lineage_events,
        lineage_basis,
        derived_indexes: metadata.derived_indexes,
        allocations: ManifestMetadataAllocations {
            _coverage: memory,
            ..Default::default()
        },
    };
    Ok((draft, members))
}
fn overflow() -> Denial {
    Denial::Request(
        contracts::execution::denial_cause::RelationalExecutionDenialCause::MemoryOverflow,
    )
}
