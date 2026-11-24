//! Minimal owned read context for exact checkpoint-image verification.
use crate::branch::RelationalBranchRoot;
use crate::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use crate::history::data::BranchId;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

/// Existing projection read handles and scalar observation metadata only.
/// This owns no runtime, configuration lock, mutation port or reconstruction grant.
#[derive(Debug)]
pub(crate) struct CheckpointProjectionContext {
    runtime_identity: u64,
    current_version: crate::identity::data::VersionId,
    instrumentation:
        crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    symbols: crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
}
pub(crate) struct RequestOwnedCheckpointProjection {
    view: VisibilityProjectionView,
    _memory: Option<ExecutionMemoryReservation>,
}
impl RequestOwnedCheckpointProjection {
    pub(crate) fn view(&self) -> &VisibilityProjectionView {
        &self.view
    }
}
impl CheckpointProjectionContext {
    pub(crate) fn from_read_handles(
        _lane: &ReconstructionLane<'_>,
        runtime_identity: u64,
        current_version: crate::identity::data::VersionId,
        instrumentation: crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
        symbols: crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Self {
        Self {
            runtime_identity,
            current_version,
            instrumentation,
            symbols,
        }
    }
    pub(crate) fn symbols(
        &self,
    ) -> crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable {
        self.symbols.clone()
    }
    pub(crate) fn project(
        &self,
        _lane: &ReconstructionLane<'_>,
        request: &ExecutionRequest<'_, '_>,
        branch: &BranchId,
        root: Arc<RelationalBranchRoot>,
    ) -> Result<RequestOwnedCheckpointProjection, Stop> {
        // The root is already admitted by its actual source owner. This operation
        // copies the observing branch name and shares that immutable root/schema.
        crate::durability::data::recovery_stop::account_work(request, branch.0.len() as u64 + 1)?;
        let memory = request
            .reserve_retained_memory(branch.0.len() as u64)
            .map_err(|cause| Stop::Request(cause.into()))?;
        let schema = super::schema_custody::ProjectionSchemaCustody::Published(
            root.retained_schema_authority(),
        );
        // A readmitted root keeps its commit receipt even when its original
        // envelope is not resident; the receipt names the same version.
        let version = root
            .commit_receipt()
            .map(|receipt| receipt.version_id)
            .ok_or_else(|| {
                crate::durability::data::recovery_errors::DurabilityError::corrupt_checkpoint(
                    "index checkpoint root has no commit",
                )
            })?;
        let basis = crate::visibility::snapshot_states::SnapshotStateBasis::Exact(
            crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_applied_root(
                branch.clone(),
                version,
                root,
            ),
        );
        Ok(RequestOwnedCheckpointProjection {
            view: VisibilityProjectionView {
                runtime_identity: self.runtime_identity,
                current_version: self.current_version,
                instrumentation: self.instrumentation.clone(),
                symbols: self.symbols.clone(),
                schema,
                resident: None,
                basis,
            },
            _memory: memory,
        })
    }
}
