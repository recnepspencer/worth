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
}

impl WorthQueryApplicationPreviewRequest {
    pub fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationPreviewReadmissionDenial {
    StaleSource,
    ForeignObservation,
    ForeignRuntime,
    SourceUnavailable,
    AdmissionRejected,
    CleanupRejected,
}

pub struct WorthQueryApplicationPreviewSession {
    handle: Option<BridgeSpeculativeSessionHandle>,
    source:
        Option<std::sync::Arc<worth_relational::facade::bridge::RelationalBridgeObservationLease>>,
    runtime_authority: u64,
    source_commit: CompositeCommitIdentity,
}

pub struct WorthQueryReadmittedApplicationPreview {
    handle: Option<BridgeSpeculativeSessionHandle>,
    _source: std::sync::Arc<worth_relational::facade::bridge::RelationalBridgeObservationLease>,
}

impl WorthQueryApplicationPreviewSession {
    pub fn readmit<Schema, Program>(
        mut self,
        runtime: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<WorthQueryReadmittedApplicationPreview, WorthQueryApplicationPreviewReadmissionDenial>
    where
        Schema: ApplicationSchema,
        Program: ApplicationProgramDefinition<Schema>,
    {
        if runtime.runtime().runtime.authority_identity().as_u64() != self.runtime_authority {
            self.discard_active()
                .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::CleanupRejected)?;
            return Err(WorthQueryApplicationPreviewReadmissionDenial::ForeignRuntime);
        }
        let current = match runtime.on_branch(runtime.current_world()).select() {
            Ok(current) => current,
            Err(_) => {
                self.discard_active()
                    .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::CleanupRejected)?;
                return Err(WorthQueryApplicationPreviewReadmissionDenial::SourceUnavailable);
            }
        };
        if current.product().selected_commit() != &self.source_commit {
            self.discard_active()
                .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::CleanupRejected)?;
            return Err(WorthQueryApplicationPreviewReadmissionDenial::StaleSource);
        }
        Ok(WorthQueryReadmittedApplicationPreview {
            handle: self.handle.take(),
            _source: self
                .source
                .take()
                .expect("an active preview session retains its exact source basis"),
        })
    }

    pub fn discard(mut self) -> Result<(), worth_runtime_bridge::facade::BridgeSpeculationError> {
        self.discard_active()
    }

    fn discard_active(
        &mut self,
    ) -> Result<(), worth_runtime_bridge::facade::BridgeSpeculationError> {
        discard_handle(
            self.handle
                .take()
                .expect("an active preview session owns its bridge handle"),
        )
    }
}

impl WorthQueryReadmittedApplicationPreview {
    pub fn promote(
        mut self,
    ) -> Result<
        BridgeSpeculativePromotionOutcome,
        worth_runtime_bridge::facade::BridgeSpeculationError,
    > {
        self.handle
            .take()
            .expect("a readmitted preview owns its bridge handle")
            .promote()
    }

    pub fn discard(mut self) -> Result<(), worth_runtime_bridge::facade::BridgeSpeculationError> {
        discard_handle(
            self.handle
                .take()
                .expect("a readmitted preview owns its bridge handle"),
        )
    }
}

impl Drop for WorthQueryApplicationPreviewSession {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = discard_handle(handle);
        }
    }
}

impl Drop for WorthQueryReadmittedApplicationPreview {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = discard_handle(handle);
        }
    }
}

fn discard_handle(
    handle: BridgeSpeculativeSessionHandle,
) -> Result<(), worth_runtime_bridge::facade::BridgeSpeculationError> {
    handle
        .discard(vec![BridgePreviewResidueClass::TemporaryDiagnosticsResidue])
        .map(|_| ())
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
        let selected = self
            .select_application_read_observation(observation)
            .map_err(|denial| match denial {
                crate::basis::WorthQueryProductBranchAdmissionDenial::ForeignOwner => {
                    WorthQueryApplicationPreviewReadmissionDenial::ForeignObservation
                }
                _ => WorthQueryApplicationPreviewReadmissionDenial::SourceUnavailable,
            })?;
        let source = selected.product().bridge_source();
        let snapshot = source.snapshot_identity().clone();
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
                1,
                1,
                0,
            ))
            .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::AdmissionRejected)?;
        Ok(WorthQueryApplicationPreviewSession {
            handle: Some(handle),
            source: Some(source),
            runtime_authority: self.runtime().runtime.authority_identity().as_u64(),
            source_commit: observation.selected_commit().clone(),
        })
    }
}
