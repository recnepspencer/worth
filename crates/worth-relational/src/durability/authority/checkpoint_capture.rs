use crate::branch::root_checkpoint::RelationalBranchRootCheckpoint;
use crate::durability::data::DurabilityError;
use crate::history::commit::RelationalCommitArtifact;
use crate::history::data::BranchId;
use crate::history::data::{PositionedCanonicalCommit, RelationalCommitReceipt};
use crate::history::retention::RetainedIndexRoots;
use std::sync::Arc;

pub(super) mod runtime_selection;

pub struct CapturedCheckpointBasis {
    pub(super) recorded_components: std::collections::BTreeMap<
        crate::history::data::CommitId,
        Vec<crate::branch::basis_descriptor::RelationalBranchBasisDescriptor>,
    >,
    pub(super) retained_empty_components:
        Vec<crate::durability::data::RelationalCheckpointEmptyComponent>,
    pub(super) empty_component_schemas: Vec<crate::durability::data::DurableBranchRootSchemaImage>,
    pub(super) component_holds: Vec<crate::branch::basis::AdmittedRelationalBranchBasis>,
    pub(super) latest_commit: Option<RelationalCommitReceipt>,
    pub(super) selected_directory:
        Option<std::sync::Arc<crate::durability::data::RelationalCheckpointDirectory>>,
    pub(super) branch_cells:
        Vec<crate::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint>,
    pub(super) branch_lifecycle: Vec<crate::durability::data::DurableBranchLifecycleImage>,
    pub(super) retired_branch_names: Vec<crate::history::data::BranchId>,
    pub(super) covered_transitions:
        crate::branch::transition::sequence::RelationalBranchTransitionSequence,
    pub(super) branch_roots: Vec<RelationalBranchRootCheckpoint>,
    pub(super) record_identity: CapturedRecordIdentity,
    pub(super) envelopes: Vec<PositionedCanonicalCommit>,
    pub(super) envelope_cells:
        Vec<std::sync::Arc<crate::history::commit::RelationalCommitEnvelopeResidency>>,
    pub(super) journal_index: Vec<crate::history::commit::RelationalCommitIndexEntry>,
    pub(super) lineage_basis: crate::lineage::data::LineageCheckpointDigestBasis,
    pub(super) commit_descriptors: std::collections::BTreeMap<
        crate::history::data::CommitId,
        crate::branch::target::RelationalBranchRootDescriptor,
    >,
    pub(super) partitions:
        crate::runtime::state::subsystems::storage::partition_edition::PartitionEdition,
    pub(super) aspect_contracts: crate::schema::data::AspectContractPlanCatalog,
    pub(super) mirror_schema_authority:
        Option<crate::branch::root::schema::RelationalBranchRootSchemaAuthority>,
    pub(super) lineage_nodes: Vec<crate::lineage::data::LineageNode>,
    pub(super) relation_lineages: Vec<(
        crate::identity::data::RelationId,
        crate::identity::data::LineageId,
    )>,
    pub(super) lineage_events: Vec<crate::durability::data::RelationalCheckpointLineageEventRow>,
    pub(super) lineage_event_pages_resident: bool,
    pub(super) lineage_event_leases:
        Vec<crate::runtime::state::subsystems::lineage::ResidentLineageEventPage>,
    pub(super) guarded_member_sources: std::collections::BTreeMap<
        crate::durability::medium::RelationalDurableMemberLocator,
        (
            crate::storage::data::RelationalStorageSubject,
            std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        ),
    >,
    pub(super) index_state: crate::runtime::state::subsystems::indexing::IndexingState,
    pub(super) index_original: Option<
        std::sync::Arc<super::super::derived_index_artifacts::cold_checkpoint::DeferredIndexImage>,
    >,
    pub(super) retained_indexes: crate::history::retention::RetainedIndexRoots,
    pub(super) index_definitions: Vec<crate::indexes::data::DerivedIndexDefinition>,
    pub(super) carried_index_image:
        Option<crate::durability::medium::RelationalDurableMemberLocator>,
    pub(super) derived_index_checkpoint:
        crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts,
    pub(super) symbol_table: crate::symbols::data::SymbolTableSnapshot,
    pub(super) symbol_identity_frontier: u32,
    pub(super) symbol_pages_resident: bool,
    pub(super) runtime_name: String,
}

pub(super) type CapturedRecordIdentity = crate::durability::data::DurableRecordIdentityState;

pub(super) fn captured_lineage(
    published: Vec<(
        Arc<crate::lineage::data::LineageEventRecord>,
        crate::history::data::CommitId,
    )>,
    lineage_event_leases: &[crate::runtime::state::subsystems::lineage::ResidentLineageEventPage],
) -> Result<Vec<crate::durability::data::RelationalCheckpointLineageEventRow>, DurabilityError> {
    let mut lineage_events = published
        .into_iter()
        .map(|(event, publication_commit_id)| {
            (
                event.event_id(),
                crate::durability::data::RelationalCheckpointLineageEventRow {
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

pub(super) fn capture_root_links(
    artifacts: &[Arc<RelationalCommitArtifact>],
    branch_roots: &[RelationalBranchRootCheckpoint],
    branch_cells: &[crate::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint],
    main: &BranchId,
) -> Result<
    (
        RetainedIndexRoots,
        Option<crate::branch::root::schema::RelationalBranchRootSchemaAuthority>,
    ),
    DurabilityError,
> {
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
                        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
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
    let mut retained = crate::history::retention::RetainedIndexRoots::default();
    for cell in branch_cells {
        retained.latest_branches.insert(cell.branch_id.clone());
        let worth_foundational::FoundationalBranchTarget::Basis(target) = cell.observation.target()
        else {
            continue;
        };
        let (version, schema) = root_bases
            .get(&crate::history::data::CommitId(target.selected_commit_id()))
            .copied()
            .ok_or_else(|| {
                DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
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

pub(super) fn capture_index_checkpoint(
    index_state: &crate::runtime::state::subsystems::indexing::IndexingState,
    retained: &RetainedIndexRoots,
) -> Result<
    crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts,
    DurabilityError,
> {
    crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
        crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(
            lane,
            index_state.retained_generations(lane, retained),
            index_state.basis_checkpoint(lane, retained),
        )
    })
}
