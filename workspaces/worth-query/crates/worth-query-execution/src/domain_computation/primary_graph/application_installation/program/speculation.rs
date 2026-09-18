use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_bridge::facade::{
    BridgePreviewResidueClass, BridgePreviewRetainedArtifactSchema,
    BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity,
    BridgePreviewSessionIdentity, BridgeRequestKind, BridgeSignalBranchIdentity,
    BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeSpeculativeBranchBinding,
    BridgeSpeculativeBranchBindingIdentity, BridgeSpeculativePromotionOutcome,
    BridgeSpeculativeSessionHandle, BridgeSpeculativeSessionRequest, BridgeTruthViewSelector,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationPreviewRequest {
    identity: String,
    preview_artifact_count: usize,
    destroyable_artifact_count: usize,
    retained_non_authoritative_artifact_count: usize,
}

impl WorthQueryApplicationPreviewRequest {
    pub fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            preview_artifact_count: 1,
            destroyable_artifact_count: 1,
            retained_non_authoritative_artifact_count: 0,
        }
    }

    pub fn with_artifact_counts(
        mut self,
        preview: usize,
        destroyable: usize,
        retained_non_authoritative: usize,
    ) -> Self {
        self.preview_artifact_count = preview;
        self.destroyable_artifact_count = destroyable;
        self.retained_non_authoritative_artifact_count = retained_non_authoritative;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationPreviewReadmissionDenial {
    StaleSource,
    ForeignObservation,
    ForeignRuntime,
    MissingSourceBasis,
    SourceUnavailable,
    AdmissionRejected,
    CleanupRejected,
}

pub struct WorthQueryApplicationPreviewSession {
    handle: BridgeSpeculativeSessionHandle,
    runtime_authority: u64,
    source_commit: CompositeCommitIdentity,
}

pub struct WorthQueryReadmittedApplicationPreview {
    handle: BridgeSpeculativeSessionHandle,
}

impl WorthQueryApplicationPreviewSession {
    pub fn readmit<Schema, Program>(
        self,
        runtime: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<WorthQueryReadmittedApplicationPreview, WorthQueryApplicationPreviewReadmissionDenial>
    where
        Schema: ApplicationSchema,
        Program: ApplicationProgramDefinition<Schema>,
    {
        if runtime.runtime().runtime.authority_identity().as_u64() != self.runtime_authority {
            self.discard()
                .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::CleanupRejected)?;
            return Err(WorthQueryApplicationPreviewReadmissionDenial::ForeignRuntime);
        }
        let current = match runtime.on_branch(runtime.current_world()).select() {
            Ok(current) => current,
            Err(_) => {
                self.discard()
                    .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::CleanupRejected)?;
                return Err(WorthQueryApplicationPreviewReadmissionDenial::SourceUnavailable);
            }
        };
        if current.product().selected_commit() != &self.source_commit {
            self.discard()
                .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::CleanupRejected)?;
            return Err(WorthQueryApplicationPreviewReadmissionDenial::StaleSource);
        }
        Ok(WorthQueryReadmittedApplicationPreview {
            handle: self.handle,
        })
    }

    pub fn discard(self) -> Result<(), worth_runtime_bridge::facade::BridgeSpeculationError> {
        self.handle
            .discard(vec![BridgePreviewResidueClass::TemporaryDiagnosticsResidue])
            .map(|_| ())
    }

    pub fn liveness_observer(
        &self,
    ) -> worth_runtime_bridge::facade::BridgePreviewSessionLivenessObserver {
        self.handle.liveness_observer()
    }
}

impl WorthQueryReadmittedApplicationPreview {
    pub fn promote(
        self,
    ) -> Result<
        BridgeSpeculativePromotionOutcome,
        worth_runtime_bridge::facade::BridgeSpeculationError,
    > {
        self.handle.promote()
    }

    pub fn discard(self) -> Result<(), worth_runtime_bridge::facade::BridgeSpeculationError> {
        self.handle
            .discard(vec![BridgePreviewResidueClass::TemporaryDiagnosticsResidue])
            .map(|_| ())
    }
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn begin_preview(
        &self,
        observation: &WorthQueryApplicationReadObservation,
        request: WorthQueryApplicationPreviewRequest,
    ) -> Result<WorthQueryApplicationPreviewSession, WorthQueryApplicationPreviewReadmissionDenial>
    {
        self.select_application_read_observation(observation)
            .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::ForeignObservation)?;
        let snapshot = observation
            .bridge_snapshot_identity()
            .cloned()
            .ok_or(WorthQueryApplicationPreviewReadmissionDenial::MissingSourceBasis)?;
        let identity = request.identity;
        let declaration = BridgePreviewSessionDeclaration::new(
            BridgePreviewSessionDeclarationIdentity::from_stable_name(format!(
                "{identity}:declaration"
            )),
            BridgeRequestKind::Preview,
            BridgeSpeculativeBranchBinding::new(
                BridgeSpeculativeBranchBindingIdentity::from_stable_name(format!(
                    "{identity}:binding"
                )),
                super::super::super::application_branch::primary_truth_branch_identity(),
                BridgeSignalBranchIdentity::from_stable_name(format!("{identity}:signal")),
            ),
            worth_runtime_bridge::facade::BridgePreviewSessionBasis::new(
                BridgeTruthViewSelector::branch_snapshot(
                    super::super::super::application_branch::primary_truth_branch_identity(),
                    snapshot,
                ),
                BridgeSourceCapabilitySet::new(vec![
                    BridgeSourceCapability::SnapshotRead,
                    BridgeSourceCapability::BranchRead,
                ]),
                BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
            ),
        );
        let handle = self
            .runtime()
            .bridge
            .ordinary()
            .speculate(BridgeSpeculativeSessionRequest::new(
                BridgePreviewSessionIdentity::from_stable_name(format!("{identity}:session")),
                declaration,
                request.preview_artifact_count,
                request.destroyable_artifact_count,
                request.retained_non_authoritative_artifact_count,
            ))
            .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::AdmissionRejected)?;
        Ok(WorthQueryApplicationPreviewSession {
            handle,
            runtime_authority: self.runtime().runtime.authority_identity().as_u64(),
            source_commit: observation.selected_commit().clone(),
        })
    }
}
