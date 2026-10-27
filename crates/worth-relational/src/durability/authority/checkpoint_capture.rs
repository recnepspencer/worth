use crate::branch::RelationalBranchRootCheckpoint;
use crate::capabilities::RuntimeIdentitySource;
use crate::durability::data::DurabilityError;
use crate::history::data::{
    PositionedCanonicalCommit, RecordAllocationClass, RecordAllocationOrigin,
    RelationalCommitReceipt,
};
use crate::identity::data::PartitionId;
use crate::runtime::RelationalRuntime;

pub(super) struct CapturedCheckpointBasis {
    pub(super) latest_commit: Option<RelationalCommitReceipt>,
    pub(super) selected_directory:
        Option<std::sync::Arc<crate::durability::data::RelationalCheckpointDirectory>>,
    pub(super) branch_cells: Vec<crate::branch::RelationalBranchCellCheckpoint>,
    pub(super) branch_lifecycle: Vec<crate::durability::data::DurableBranchLifecycleImage>,
    pub(super) retired_branch_names: Vec<crate::history::data::BranchId>,
    pub(super) covered_transitions: crate::branch::RelationalBranchTransitionSequence,
    pub(super) branch_roots: Vec<RelationalBranchRootCheckpoint>,
    pub(super) record_identity: CapturedRecordIdentity,
    pub(super) envelopes: Vec<PositionedCanonicalCommit>,
    pub(super) envelope_cells:
        Vec<std::sync::Arc<crate::history::RelationalCommitEnvelopeResidency>>,
    pub(super) journal_index: Vec<crate::history::RelationalCommitIndexEntry>,
    pub(super) lineage_basis: crate::lineage::data::LineageCheckpointDigestBasis,
    pub(super) commit_descriptors: std::collections::BTreeMap<
        crate::history::data::CommitId,
        crate::branch::RelationalBranchRootDescriptor,
    >,
    pub(super) partitions: crate::runtime::PartitionEdition,
    pub(super) aspect_contracts: crate::schema::data::AspectContractPlanCatalog,
    pub(super) mirror_schema_authority: Option<crate::branch::RelationalBranchRootSchemaAuthority>,
    pub(super) lineage_nodes: Vec<crate::lineage::data::LineageNode>,
    pub(super) lineage_events: Vec<crate::durability::data::RelationalCheckpointLineageEventRow>,
    pub(super) lineage_event_pages_resident: bool,
    pub(super) lineage_event_leases: Vec<crate::runtime::ResidentLineageEventPage>,
    pub(super) guarded_member_sources: std::collections::BTreeMap<
        crate::durability::medium::RelationalDurableMemberLocator,
        (
            crate::storage::data::RelationalStorageSubject,
            std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        ),
    >,
    pub(super) index_state: crate::runtime::IndexingState,
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

pub(super) struct CapturedRecordIdentity {
    pub(super) generation_high_water: Vec<(RecordAllocationClass, PartitionId, u64, u32)>,
    pub(super) reusable_slots: Vec<(RecordAllocationClass, PartitionId, usize)>,
    pub(super) append_frontiers: Vec<(RecordAllocationClass, PartitionId, usize)>,
    pub(super) pending_reservations: Vec<(
        RecordAllocationClass,
        PartitionId,
        u64,
        u32,
        RecordAllocationOrigin,
    )>,
}

impl CapturedCheckpointBasis {
    /// Select one immutable recovery basis while publication handoff is
    /// excluded. Expensive checkpoint reconstruction, encoding, and I/O occur
    /// only after the caller drops the admission guard.
    pub(super) fn capture(
        runtime: &RelationalRuntime,
        routes: &crate::runtime::RelationalCanonicalPublicationRoutes,
        selection: crate::runtime::PerformedCheckpointSelection,
    ) -> Result<Self, DurabilityError> {
        // Checkpointing is a cold lane: retire expired reader roots and their
        // derived generations before selecting bounded payloads.
        runtime.run_index_generation_reclamation_pass();
        let envelopes = selection.positioned_snapshot();
        let selected_directory = runtime.durability.checkpoint_selection().snapshot();
        let artifacts = runtime.history.commit_artifacts();
        let lineage_basis = super::checkpoint_lineage_basis::capture(
            &artifacts,
            selected_directory
                .as_ref()
                .map(|directory| &directory.manifest),
        )?;
        let journal_index = artifacts
            .iter()
            .filter_map(|artifact| {
                artifact
                    .envelope_residency()
                    .journal_index()
                    .map(|entry| entry.as_ref().clone())
            })
            .collect();
        let covered_transitions = selection.covered_transitions();
        let branch_roots = runtime
            .history
            .branch_root_checkpoints()
            .map_err(|detail| {
                DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    detail,
                )
            })?;
        let envelope_versions = envelopes
            .iter()
            .map(|envelope| (envelope.commit.commit_id, envelope.commit.version_id))
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
        let branch_cells = runtime.history().branch_cells_snapshot();
        let root_bases = branch_roots
            .iter()
            .map(|root| root.commit_id())
            .zip(root_bases)
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut retained = crate::history::retention::RetainedIndexRoots::default();
        for cell in &branch_cells {
            retained.latest_branches.insert(cell.branch_id.clone());
            let worth_foundational::FoundationalBranchTarget::Basis(target) =
                cell.observation.target()
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
        let main_branch = runtime.main_branch_identity();
        let mirror_schema_authority = branch_cells
            .iter()
            .find(|cell| &cell.branch_id == main_branch.branch_id())
            .and_then(|cell| match cell.observation.target() {
                worth_foundational::FoundationalBranchTarget::Basis(target) => branch_roots
                    .iter()
                    .find(|root| root.commit_id().0 == target.selected_commit_id()),
                _ => None,
            })
            .map(|root| root.schema_authority().clone());
        let (lineage_event_leases, lineage_event_pages_resident) = runtime
            .lineage
            .capture_resident_event_pages()
            .map_err(lineage_capture_failure)?;
        let mut lineage_events = runtime
            .lineage
            .published_rows()
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
        for page in &lineage_event_leases {
            for row in page.rows() {
                if let Some(existing) = lineage_events.get(&row.event.event_id())
                    && existing != row
                {
                    return Err(lineage_capture_failure(page.binding_failure()));
                }
                lineage_events
                    .entry(row.event.event_id())
                    .or_insert_with(|| row.clone());
            }
        }
        let index_state = runtime.indexes.snapshot();
        let derived_index_checkpoint =
            super::super::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(
                index_state.retained_generations(&retained),
            )?;
        let captured = Self {
            envelope_cells: artifacts
                .iter()
                .map(|artifact| artifact.envelope_residency())
                .collect(),
            journal_index,
            lineage_basis,
            latest_commit: selection.latest_receipt().or_else(|| {
                selected_directory.as_ref().and_then(|directory| {
                    directory
                        .manifest
                        .commits
                        .last()
                        .map(|entry| entry.commit.clone())
                })
            }),
            selected_directory,
            branch_cells,
            branch_lifecycle: runtime.history.branch_lifecycle_snapshot(),
            retired_branch_names: runtime.history.retired_branch_names(),
            covered_transitions,
            branch_roots,
            record_identity: CapturedRecordIdentity {
                generation_high_water: runtime.record_identity.generation_snapshot(),
                reusable_slots: runtime.record_identity.reusable_snapshot(),
                append_frontiers: runtime.record_identity.frontier_snapshot(),
                pending_reservations: runtime.record_identity.pending_snapshot(),
            },
            commit_descriptors: artifacts
                .iter()
                .map(|artifact| (artifact.identity().commit_id(), artifact.roots().clone()))
                .collect(),
            envelopes,
            partitions: runtime.acquire_partition_edition(),
            mirror_schema_authority,
            aspect_contracts: runtime
                .schema_contract_runtime
                .aspect_contract_plans
                .clone(),
            lineage_nodes: runtime.lineage_access().resident_nodes_snapshot(),
            lineage_event_pages_resident,
            lineage_event_leases,
            guarded_member_sources: runtime
                .lineage
                .event_member_sources()
                .into_iter()
                .chain(runtime.lineage.node_export_sources())
                .chain(runtime.services.symbols.checkpoint_export_sources())
                .chain(runtime.indexes.checkpoint_export_sources())
                .collect(),
            lineage_events: lineage_events.into_values().collect(),
            index_definitions: runtime.index_access().definitions_snapshot(),
            carried_index_image: runtime.indexes.carried_checkpoint_image(),
            index_state,
            retained_indexes: retained,
            derived_index_checkpoint,
            symbol_table: runtime.services.symbols.snapshot(),
            symbol_identity_frontier: runtime
                .services
                .symbols
                .with_read(|symbols| symbols.identity_frontier()),
            symbol_pages_resident: runtime.services.symbols.checkpoint_contents_resident(),
            runtime_name: runtime.runtime_name().to_string(),
        };
        routes
            .validate_checkpoint_selection(&selection)
            .map_err(super::checkpointing::checkpoint_admission_error)?;
        Ok(captured)
    }
}

fn lineage_capture_failure(
    failure: crate::storage::data::RelationalStorageUnavailable,
) -> DurabilityError {
    let crate::durability::data::RelationalRecoveryStop::Durability(error) = failure.into() else {
        unreachable!()
    };
    error
}
