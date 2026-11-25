//! Read-only inspection of captured checkpoint and recovery evidence.

mod carried_index_inspection;
use crate::durability::authority::CompleteRelationalNativeCheckpoint;
use crate::durability::data::RelationalNativeCheckpoint;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::history::data::coordinates::{BranchId, CommitId};
use contracts::identity::data::primitives::VersionId;
use contracts::indexes::data::definition::{DerivedIndexGenerationId, DerivedIndexId};
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::storage::residency::history_capture_bound::{
    HISTORY_CAPTURE_INPUT_TARGET, HISTORY_CAPTURE_MAX_LAYERS,
};

/// Read-only history-capture fold bounds of one checkpoint directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalHistoryFoldBounds {
    pub max_layers: usize,
    pub max_input_bytes: u64,
    pub max_layers_limit: usize,
    pub input_target_bytes: u64,
    pub all_within_bounds: bool,
}
/// One carried index selection basis decoded from a checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalCompleteCarriedIndexBasis {
    pub branch: BranchId,
    pub commit: CommitId,
    pub version: VersionId,
    pub bindings: Vec<(DerivedIndexId, DerivedIndexGenerationId)>,
}
/// Read-only checkpoint index-selection codec inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalCompleteIndexSelectionInspection {
    pub format_version: u16,
    pub expected_format_version: u16,
    pub legacy_artifact_count: usize,
    pub all_bases_beside_roots: bool,
    pub bases: Vec<RelationalCompleteCarriedIndexBasis>,
}
impl crate::durability::data::RecoveryPlan {
    pub fn tail_log_len(&self) -> usize {
        self.tail_log.len()
    }
}

impl crate::durability::data::DurableCheckpoint {
    pub fn has_cold_directory(&self) -> bool {
        self.directory.is_some()
    }
    pub fn branch_root_count(&self) -> usize {
        self.directory
            .as_ref()
            .map_or(self.branch_roots.len(), |dir| dir.manifest.roots.len())
    }
}

impl RelationalNativeCheckpoint {
    /// Decode this capture's directory and report its history-capture fold bounds.
    pub fn inspect_history_fold_bounds(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: Arc<dyn RelationalDurableColdSource>,
    ) -> Result<Option<RelationalHistoryFoldBounds>, RelationalRecoveryStop> {
        let decoded =
            crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
                execution,
                self.bytes(),
                source,
            )?;
        let checkpoint = decoded.checkpoint;
        let directory = checkpoint
            .directory
            .as_ref()
            .or(checkpoint.canonical_directory.as_ref());
        Ok(directory.map(|directory| {
            let mut max_layers = 0usize;
            let mut max_input_bytes = 0u64;
            let mut all_within_bounds = true;
            for leaf in directory
                .manifest
                .roots
                .iter()
                .flat_map(|root| root.regions.iter())
                .chain(directory.manifest.mirror.iter())
            {
                let (layers, input) = directory.history_members.fold_span(leaf.history);
                max_layers = max_layers.max(layers);
                max_input_bytes = max_input_bytes.max(input);
                if layers > HISTORY_CAPTURE_MAX_LAYERS || input >= HISTORY_CAPTURE_INPUT_TARGET {
                    all_within_bounds = false;
                }
            }
            RelationalHistoryFoldBounds {
                max_layers,
                max_input_bytes,
                max_layers_limit: HISTORY_CAPTURE_MAX_LAYERS,
                input_target_bytes: HISTORY_CAPTURE_INPUT_TARGET,
                all_within_bounds,
            }
        }))
    }
    /// Decode this capture's carried index-selection payload (read-only).
    pub fn inspect_carried_index_selections(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: Arc<dyn RelationalDurableColdSource>,
    ) -> Result<Option<RelationalCompleteIndexSelectionInspection>, RelationalRecoveryStop> {
        carried_index_inspection::inspect_carried_index_selections(self, execution, source)
    }
}

impl CompleteRelationalNativeCheckpoint {
    /// Decode this Complete capture's carried index-selection payload (read-only).
    pub fn inspect_carried_index_selections(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: Arc<dyn RelationalDurableColdSource>,
    ) -> Result<Option<RelationalCompleteIndexSelectionInspection>, RelationalRecoveryStop> {
        self.checkpoint()
            .inspect_carried_index_selections(execution, source)
    }
}
