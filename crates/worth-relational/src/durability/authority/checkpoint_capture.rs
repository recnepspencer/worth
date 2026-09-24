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
    pub(super) branch_cells: Vec<crate::branch::RelationalBranchCellCheckpoint>,
    pub(super) branch_roots: Vec<RelationalBranchRootCheckpoint>,
    pub(super) record_identity: CapturedRecordIdentity,
    pub(super) envelopes: Vec<PositionedCanonicalCommit>,
    pub(super) partitions: Vec<crate::storage::overlay::PartitionState>,
    pub(super) aspect_contracts: crate::schema::data::AspectContractPlanCatalog,
    pub(super) lineage_nodes: Vec<crate::lineage::data::LineageNode>,
    pub(super) index_definitions: Vec<crate::indexes::data::DerivedIndexDefinition>,
    pub(super) derived_index_checkpoint:
        crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts,
    pub(super) symbol_table: crate::symbols::data::SymbolTableSnapshot,
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
        let captured = Self {
            latest_commit: envelopes
                .last()
                .map(|positioned| positioned.envelope().commit.clone()),
            branch_cells,
            branch_roots,
            record_identity: CapturedRecordIdentity {
                generation_high_water: runtime.record_identity.generation_snapshot(),
                reusable_slots: runtime.record_identity.reusable_snapshot(),
                append_frontiers: runtime.record_identity.frontier_snapshot(),
                pending_reservations: runtime.record_identity.pending_snapshot(),
            },
            envelopes,
            partitions: runtime.materialize_partitions(),
            aspect_contracts: runtime
                .schema_contract_runtime
                .aspect_contract_plans
                .clone(),
            lineage_nodes: runtime.lineage_access().nodes_snapshot(),
            index_definitions: runtime.index_access().definitions_snapshot(),
            derived_index_checkpoint:
                super::super::derived_index_artifacts::checkpoint_derived_index_artifacts(
                    runtime, &retained,
                )?,
            symbol_table: runtime.services.symbols.snapshot(),
            runtime_name: runtime.runtime_name().to_string(),
        };
        routes
            .validate_checkpoint_selection(&selection)
            .map_err(super::checkpointing::checkpoint_admission_error)?;
        Ok(captured)
    }
}
