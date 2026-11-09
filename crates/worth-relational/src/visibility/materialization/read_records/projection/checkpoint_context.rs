//! Minimal owned read context for exact checkpoint-image verification.
use crate::branch::RelationalBranchRoot;
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::history::data::BranchId;
use crate::runtime::{RelationalRuntime, VisibilityProjectionView};
use crate::storage::substrate::maintenance::ReconstructionLane;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

/// Existing projection read handles and scalar observation metadata only.
/// This owns no runtime, configuration lock, mutation port or reconstruction grant.
#[derive(Debug)]
pub(crate) struct CheckpointProjectionContext {
    runtime_identity: u64,
    current_version: crate::identity::data::VersionId,
    instrumentation: crate::runtime::RuntimeInstrumentation,
    symbols: crate::runtime::RuntimeSymbolTable,
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
    pub(crate) fn capture(_lane: &ReconstructionLane<'_>, runtime: &RelationalRuntime) -> Self {
        Self {
            runtime_identity: runtime.runtime_instance_id(),
            current_version: runtime.current_version_id(),
            instrumentation: runtime.services.instrumentation.clone(),
            symbols: runtime.services.symbols.clone(),
        }
    }
    pub(crate) fn symbols(&self) -> crate::runtime::RuntimeSymbolTable {
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
        crate::durability::artifact_payload_emission::request::account_work(
            request,
            branch.0.len() as u64 + 1,
        )?;
        let memory = request
            .reserve_retained_memory(branch.0.len() as u64)
            .map_err(|cause| Stop::Request(cause.into()))?;
        let schema = super::schema_custody::ProjectionSchemaCustody::Published(
            root.retained_schema_authority(),
        );
        let basis = crate::visibility::snapshot_states::SnapshotStateBasis::Exact(
            crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_prepared_root(
                branch.clone(),
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
