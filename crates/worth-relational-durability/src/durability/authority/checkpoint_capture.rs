use contracts::durability::data::recovery_errors::DurabilityError;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::BranchId;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::root_checkpoint::RelationalBranchRootCheckpoint;
use worth_relational_branch::internal_port::history::commit::artifact::RelationalCommitArtifact;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use worth_relational_branch::internal_port::history::retention::reclamation::RetainedIndexRoots;
use worth_relational_contracts::internal_port as contracts;

pub struct CapturedCheckpointBasis {
    pub recorded_components: std::collections::BTreeMap<
        contracts::history::data::coordinates::CommitId,
        Vec<worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor>,
    >,
    pub retained_empty_components:
        Vec<crate::durability::data::RelationalCheckpointEmptyComponent>,
    pub empty_component_schemas: Vec<crate::durability::data::DurableBranchRootSchemaImage>,
    pub component_holds: Vec<worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis>,
    pub latest_commit: Option<RelationalCommitReceipt>,
    pub selected_directory:
        Option<std::sync::Arc<crate::durability::data::RelationalCheckpointDirectory>>,
    pub branch_cells:
        Vec<contracts::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint>,
    pub branch_lifecycle: Vec<worth_relational_branch::internal_port::durability::data::branch_lifecycle_images::DurableBranchLifecycleImage>,
    pub retired_branch_names: Vec<contracts::history::data::coordinates::BranchId>,
    pub covered_transitions:
        contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    pub branch_roots: Vec<RelationalBranchRootCheckpoint>,
    pub record_identity: CapturedRecordIdentity,
    pub carried_record_identity: Option<
        std::sync::Arc<
            worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::paged_base::PagedIdentityBase,
        >,
    >,
    pub envelopes: Vec<PositionedCanonicalCommit>,
    pub envelope_cells:
        Vec<std::sync::Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>>,
    pub journal_index: Vec<worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry>,
    pub lineage_basis: contracts::lineage::data::artifacts::checkpoint_artifact::LineageCheckpointDigestBasis,
    pub commit_descriptors: std::collections::BTreeMap<
        contracts::history::data::coordinates::CommitId,
        contracts::branch::target::RelationalBranchRootDescriptor,
    >,
    pub partitions:
        worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionEdition,
    pub aspect_contracts: contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
    pub mirror_schema_authority:
        Option<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>,
    pub lineage_nodes: Vec<contracts::lineage::data::graph::LineageNode>,
    pub relation_lineages: Vec<(
        contracts::identity::data::record_ids::RelationId,
        contracts::identity::data::primitives::LineageId,
    )>,
    pub lineage_events: Vec<worth_relational_lineage::internal_port::durability::data::lineage_event_pages::RelationalCheckpointLineageEventRow>,
    pub lineage_event_pages_resident: bool,
    pub lineage_event_leases:
        Vec<worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::event_residency::ResidentLineageEventPage>,
    pub guarded_member_sources: std::collections::BTreeMap<
        worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator,
        (
            contracts::storage::data::storage_unavailable::RelationalStorageSubject,
            std::sync::Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
        ),
    >,
    pub index_state: worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexingState,
    pub index_original: Option<
        std::sync::Arc<super::super::derived_index_artifacts::cold_checkpoint::DeferredIndexImage>,
    >,
    pub retained_indexes: worth_relational_branch::internal_port::history::retention::reclamation::RetainedIndexRoots,
    pub index_definitions: Vec<contracts::indexes::data::definition::DerivedIndexDefinition>,
    pub carried_index_image:
        Option<worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator>,
    pub derived_index_checkpoint:
        crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts,
    pub symbol_table: contracts::symbols::data::symbol_table_snapshot::SymbolTableSnapshot,
    pub symbol_identity_frontier: u32,
    pub symbol_pages_resident: bool,
    pub runtime_name: String,
}

pub type CapturedRecordIdentity = worth_relational_storage::internal_port::storage::data::record_identity_image::DurableRecordIdentityState;

pub fn captured_lineage(
    published: Vec<(
        Arc<contracts::lineage::data::events::LineageEventRecord>,
        contracts::history::data::coordinates::CommitId,
    )>,
    lineage_event_leases: &[worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::event_residency::ResidentLineageEventPage],
) -> Result<Vec<worth_relational_lineage::internal_port::durability::data::lineage_event_pages::RelationalCheckpointLineageEventRow>, DurabilityError>{
    let mut lineage_events = published
        .into_iter()
        .map(|(event, publication_commit_id)| {
            (
                event.event_id(),
                worth_relational_lineage::internal_port::durability::data::lineage_event_pages::RelationalCheckpointLineageEventRow {
                    event,
                    publication_commit_id,
                },
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    for page in lineage_event_leases {
        for row in page.rows() {
            if let Some(existing) = lineage_events.get(&row.event.event_id())
                && existing != row
            {
                return Err(DurabilityError::from(page.binding_failure()));
            }
            lineage_events
                .entry(row.event.event_id())
                .or_insert_with(|| row.clone());
        }
    }

    Ok(lineage_events.into_values().collect())
}

pub fn capture_root_links(
    artifacts: &[Arc<RelationalCommitArtifact>],
    branch_roots: &[RelationalBranchRootCheckpoint],
    branch_cells: &[contracts::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint],
    main: &BranchId,
) -> Result<
    (
        RetainedIndexRoots,
        Option<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>,
    ),
    DurabilityError,
>{
    let envelope_versions = artifacts
        .iter()
        .map(|artifact| {
            let receipt = artifact.receipt();
            (receipt.commit_id, receipt.version_id)
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let root_bases = branch_roots
        .iter()
        .map(|root| {
            envelope_versions
                .get(&root.commit_id())
                .copied()
                .map(|version| (version, root.schema_authority().schema_version()))
                .ok_or_else(|| {
                    DurabilityError::new(
                        contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                        "branch root has no selected canonical envelope",
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let root_bases = branch_roots
        .iter()
        .map(|root| root.commit_id())
        .zip(root_bases)
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut retained = worth_relational_branch::internal_port::history::retention::reclamation::RetainedIndexRoots::default();
    for cell in branch_cells {
        retained.latest_branches.insert(cell.branch_id.clone());
        let worth_foundational::FoundationalBranchTarget::Basis(target) = cell.observation.target()
        else {
            continue;
        };
        let (version, schema) = root_bases
            .get(&contracts::history::data::coordinates::CommitId(target.selected_commit_id()))
            .copied()
            .ok_or_else(|| {
                DurabilityError::new(
                    contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                    "branch cell has no selected root image",
                )
            })?;
        retained
            .live
            .insert((cell.branch_id.clone(), version, schema));
    }
    let mirror_schema_authority = branch_cells
        .iter()
        .find(|cell| &cell.branch_id == main)
        .and_then(|cell| match cell.observation.target() {
            worth_foundational::FoundationalBranchTarget::Basis(target) => branch_roots
                .iter()
                .find(|root| root.commit_id().0 == target.selected_commit_id()),
            _ => None,
        })
        .map(|root| root.schema_authority().clone());
    Ok((retained, mirror_schema_authority))
}

pub fn capture_index_checkpoint(
    index_state: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexingState,
    retained: &RetainedIndexRoots,
) -> Result<
    crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts,
    DurabilityError,
> {
    contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
        crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(
            lane,
            index_state.retained_generations(lane, retained),
            index_state.basis_checkpoint(lane, retained),
        )
    })
}
