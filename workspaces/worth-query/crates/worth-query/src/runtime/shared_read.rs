use std::sync::Arc;

use crate::authorized_projection::AuthorizedProjectionArtifact;
use crate::canonicalization::CanonicalResultShapeArtifact;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::projection_consumption::{
    ProjectionAuthorityContract, ProjectionAuthorityOutcome, ProjectionFactConsumptionPathError,
};

use super::published_artifacts::{
    WorthQueryPublishedArtifactDiagnostics, WorthQueryPublishedArtifactEntry,
    WorthQueryPublishedArtifactRegistry, WorthQueryPublishedArtifactResolution,
};
use super::{
    WorthQueryDerivedArtifactBinding, WorthQueryDerivedMaterializationTarget,
    WorthQueryDerivedViewHandle, WorthQueryRuntime, WorthQueryRuntimeAsyncResultState,
    WorthQueryRuntimeError, WorthQuerySharedReadGenerationLease,
    WorthQuerySharedReadPinningDiagnostics,
};

#[derive(Debug)]
pub enum WorthQueryPublishedProjectionAuthorityOutcome {
    Current(ProjectionAuthorityOutcome),
    ResultState(WorthQueryRuntimeAsyncResultState),
}

impl WorthQueryPublishedProjectionAuthorityOutcome {
    pub fn authority(
        &self,
    ) -> Option<&crate::projection_consumption::WorthQueryConsumedProjectionAuthority> {
        match self {
            Self::Current(outcome) => outcome.authority(),
            Self::ResultState(_) => None,
        }
    }

    pub fn result_state(&self) -> Option<&WorthQueryRuntimeAsyncResultState> {
        match self {
            Self::Current(_) => None,
            Self::ResultState(state) => Some(state),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublishedProjectionInspection {
    published: bool,
    artifact_binding_identity: Option<WorthQueryEvidenceIdentity>,
    snapshot_identity: WorthQuerySnapshotIdentity,
    snapshot_evidence_identity: WorthQueryEvidenceIdentity,
    async_result_state: Option<WorthQueryRuntimeAsyncResultState>,
}

impl WorthQueryPublishedProjectionInspection {
    pub fn published(&self) -> bool {
        self.published
    }

    pub fn artifact_binding_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.artifact_binding_identity.as_ref()
    }

    pub fn artifact_binding_for_reporting(&self) -> Option<&str> {
        self.artifact_binding_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.snapshot_evidence_identity
    }

    pub fn async_result_state(&self) -> Option<&WorthQueryRuntimeAsyncResultState> {
        self.async_result_state.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryPublishedDerivedArtifactHandle {
    lease: WorthQuerySharedReadGenerationLease,
    view_name: String,
    published_binding: Option<Arc<WorthQueryDerivedArtifactBinding>>,
    async_result_state: Option<WorthQueryRuntimeAsyncResultState>,
}

impl WorthQueryPublishedDerivedArtifactHandle {
    pub fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        self.lease.generation().snapshot_identity()
    }

    pub fn published_binding(&self) -> Option<&WorthQueryDerivedArtifactBinding> {
        self.published_binding.as_deref()
    }

    pub fn async_result_state(&self) -> Option<&WorthQueryRuntimeAsyncResultState> {
        self.async_result_state.as_ref()
    }

    pub fn inspect_projection_consumption(&self) -> WorthQueryPublishedProjectionInspection {
        WorthQueryPublishedProjectionInspection {
            published: self.published_binding.is_some(),
            artifact_binding_identity: self
                .published_binding
                .as_ref()
                .map(|binding| binding.binding_identity().clone()),
            snapshot_identity: self.snapshot_identity().clone(),
            snapshot_evidence_identity: self.snapshot_identity().evidence_identity(),
            async_result_state: self.async_result_state.clone(),
        }
    }

    pub fn consume_projection_authority(
        &self,
        result_shape: &CanonicalResultShapeArtifact,
        authorized_projection: &AuthorizedProjectionArtifact,
        contract: ProjectionAuthorityContract,
    ) -> Result<WorthQueryPublishedProjectionAuthorityOutcome, ProjectionFactConsumptionPathError>
    {
        match &self.published_binding {
            Some(binding) => Ok(WorthQueryPublishedProjectionAuthorityOutcome::Current(
                binding.consume_projection_authority(
                    result_shape,
                    authorized_projection,
                    contract,
                )?,
            )),
            None => Ok(WorthQueryPublishedProjectionAuthorityOutcome::ResultState(
                self.async_result_state
                    .clone()
                    .expect("unpublished shared-read artifact must retain typed async posture"),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedReadBasisInspection {
    generation_ordinal: u64,
    snapshot_identity: WorthQuerySnapshotIdentity,
    snapshot_evidence_identity: WorthQueryEvidenceIdentity,
}

impl WorthQuerySharedReadBasisInspection {
    pub(in crate::runtime) fn from_lease(lease: &WorthQuerySharedReadGenerationLease) -> Self {
        let generation = lease.generation();
        Self {
            generation_ordinal: generation.ordinal(),
            snapshot_identity: generation.snapshot_identity().clone(),
            snapshot_evidence_identity: generation.snapshot_identity().evidence_identity(),
        }
    }

    pub fn generation_ordinal(&self) -> u64 {
        self.generation_ordinal
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.snapshot_evidence_identity
    }
}

#[derive(Clone, Debug)]
pub struct WorthQuerySharedReadContext {
    lease: WorthQuerySharedReadGenerationLease,
    published_artifacts: WorthQueryPublishedArtifactRegistry,
}

impl WorthQuerySharedReadContext {
    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        self.lease.generation().snapshot_identity()
    }

    pub fn inspect_basis(&self) -> WorthQuerySharedReadBasisInspection {
        WorthQuerySharedReadBasisInspection::from_lease(&self.lease)
    }

    pub fn published_derived_artifact<T>(
        &self,
        view: &WorthQueryDerivedViewHandle<T>,
    ) -> Result<WorthQueryPublishedDerivedArtifactHandle, WorthQueryRuntimeError> {
        if !self.lease.is_generation_live() {
            return Err(super::worth_query_shared_read_stale_basis_error(
                self.snapshot_identity().clone(),
            ));
        }
        let target = WorthQueryDerivedMaterializationTarget::from(view);
        match self
            .published_artifacts
            .resolve(self.lease.generation(), &target)
        {
            WorthQueryPublishedArtifactResolution::Published {
                binding,
                async_result_state,
            } => Ok(WorthQueryPublishedDerivedArtifactHandle {
                lease: self.lease.clone(),
                view_name: target.terminal_view_name_projection().to_string(),
                published_binding: Some(binding),
                async_result_state,
            }),
            WorthQueryPublishedArtifactResolution::Unpublished { async_result_state } => {
                Ok(WorthQueryPublishedDerivedArtifactHandle {
                    lease: self.lease.clone(),
                    view_name: target.terminal_view_name_projection().to_string(),
                    published_binding: None,
                    async_result_state: Some(async_result_state),
                })
            }
            WorthQueryPublishedArtifactResolution::MissingGeneration => Err(
                super::worth_query_shared_read_stale_basis_error(self.snapshot_identity().clone()),
            ),
            WorthQueryPublishedArtifactResolution::MissingView => Err(
                WorthQueryRuntimeError::MissingDerivedView(view.name().to_string()),
            ),
        }
    }
    #[cfg(test)]
    pub(in crate::runtime) fn from_runtime(runtime: &WorthQueryRuntime) -> Self {
        runtime
            .mint_shared_read_context()
            .expect("runtime-owned shared read context should mint")
    }
}

impl WorthQueryRuntime {
    pub(in crate::runtime) fn capture_shared_read_generation(
        &mut self,
        snapshot_identity: WorthQuerySnapshotIdentity,
    ) {
        self.publish_shared_read_generation(snapshot_identity);
    }

    pub(in crate::runtime) fn mint_shared_read_context(
        &self,
    ) -> Result<WorthQuerySharedReadContext, WorthQueryRuntimeError> {
        let snapshot_identity = self.current_snapshot_identity()?;
        if !self.shared_read_pins.has_current_generation() {
            self.capture_shared_read_generation_for_current_snapshot(snapshot_identity);
        }
        let lease = self
            .shared_read_pins
            .pin_current_generation()
            .ok_or_else(|| {
                WorthQueryRuntimeError::Workspace(
                    crate::memory_workspace::WorthQueryWorkspaceError::new(
                        "shared read context requires a committed generation",
                    ),
                )
            })?;
        Ok(WorthQuerySharedReadContext {
            lease,
            published_artifacts: self.published_artifacts.clone(),
        })
    }
    pub fn shared_read_counters(&self) -> super::WorthQuerySharedReadCounters {
        self.shared_read_pins
            .counters()
            .with_published_artifacts(self.published_artifacts.counters().snapshot())
    }

    pub fn shared_read_pinning_diagnostics(&self) -> WorthQuerySharedReadPinningDiagnostics {
        self.shared_read_pins.diagnostics()
    }

    pub fn published_artifact_diagnostics(&self) -> WorthQueryPublishedArtifactDiagnostics {
        self.published_artifacts.diagnostics()
    }

    pub fn invalidate_shared_read_snapshot_for_certification(
        &self,
        snapshot_identity: &WorthQuerySnapshotIdentity,
    ) {
        self.shared_read_pins
            .force_retire_snapshot_identity(snapshot_identity);
    }

    pub fn record_shared_read_hot_path_lock_for_certification(&self) {
        self.shared_read_pins
            .record_committed_read_hot_path_lock_for_certification();
    }
}

impl WorthQueryRuntime {
    fn capture_shared_read_generation_for_current_snapshot(
        &self,
        snapshot_identity: WorthQuerySnapshotIdentity,
    ) {
        self.publish_shared_read_generation(snapshot_identity);
    }

    fn publish_shared_read_generation(&self, snapshot_identity: WorthQuerySnapshotIdentity) {
        let generation = self
            .shared_read_pins
            .capture_committed_snapshot(snapshot_identity.clone());
        let entries = self
            .derived_views
            .iter()
            .map(|(target, runtime_view)| {
                let entry = WorthQueryPublishedArtifactEntry::from_runtime_view(
                    &snapshot_identity,
                    target.terminal_view_name_projection(),
                    runtime_view,
                )
                .expect("published artifact entry should package runtime view");
                (target.clone(), entry)
            })
            .collect();
        self.published_artifacts
            .publish_generation(&generation, entries);
        self.published_artifacts
            .retain_generations(&self.shared_read_pins.retained_generation_ordinals());
    }
}
