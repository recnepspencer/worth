use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
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

#[cfg(test)]
mod counted_read_stops;
#[cfg(test)]
mod discard_fault;

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
    ForeignPhase,
    SourceUnavailable,
    SourceSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    AdmissionRejected,
    CleanupRejected,
    CleanupRejectedAfterStop {
        stop: Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
        cleanup: worth_runtime_bridge::facade::BridgeSpeculationErrorKind,
    },
}

pub struct WorthQueryApplicationPreviewSession {
    handle: Option<BridgeSpeculativeSessionHandle>,
    source: Option<std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease>>,
    runtime_authority: u64,
    source_commit: CompositeCommitIdentity,
    #[cfg(test)]
    discard_fault: std::rc::Rc<std::cell::Cell<bool>>,
}

pub struct WorthQueryReadmittedApplicationPreview {
    handle: Option<BridgeSpeculativeSessionHandle>,
    _source: std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease>,
}

impl WorthQueryApplicationPreviewSession {
    pub fn readmit<Schema, Program>(
        mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<WorthQueryReadmittedApplicationPreview, WorthQueryApplicationPreviewReadmissionDenial>
    where
        Schema: ApplicationSchema,
        Program: ApplicationProgramDefinition<Schema>,
    {
        let phase_request = phase
            .execution_request_for(&runtime.runtime().product_runtime)
            .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::ForeignPhase)?;
        let execution = &phase_request;
        if runtime.runtime().runtime.authority_identity().as_u64() != self.runtime_authority {
            self.discard_active()
                .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::CleanupRejected)?;
            return Err(WorthQueryApplicationPreviewReadmissionDenial::ForeignRuntime);
        }
        let current = match runtime
            .on_branch(runtime.current_world())
            .select_with_request(execution)
        {
            Ok(current) => current,
            Err(crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(stop)) => {
                return Err(match self.discard_active() {
                    Ok(()) => WorthQueryApplicationPreviewReadmissionDenial::SourceSelection(
                        crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(stop),
                    ),
                    Err(cleanup) => {
                        WorthQueryApplicationPreviewReadmissionDenial::CleanupRejectedAfterStop {
                            stop,
                            cleanup: cleanup.kind(),
                        }
                    }
                });
            }
            Err(crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_)) => {
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
        #[cfg(test)]
        if self.discard_fault.get() {
            return self
                .handle
                .take()
                .expect("active preview owns its handle")
                .discard(vec![
                    BridgePreviewResidueClass::AuthoritativeDiagnosticsResidue,
                ])
                .map(|_| ());
        }
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
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        observation: &WorthQueryApplicationReadObservation,
        request: WorthQueryApplicationPreviewRequest,
    ) -> Result<WorthQueryApplicationPreviewSession, WorthQueryApplicationPreviewReadmissionDenial>
    {
        let phase_request = phase
            .execution_request_for(&self.runtime().product_runtime)
            .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::ForeignPhase)?;
        let execution = &phase_request;
        let selected = self
            .select_application_read_observation_with_request(execution, observation)
            .map_err(|denial| match denial {
                cause @ crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(_) => {
                    WorthQueryApplicationPreviewReadmissionDenial::SourceSelection(cause)
                }
                crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(
                    AdmissionRefusal::ForeignOwner,
                ) => WorthQueryApplicationPreviewReadmissionDenial::ForeignObservation,
                crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_) => {
                    WorthQueryApplicationPreviewReadmissionDenial::SourceUnavailable
                }
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
            #[cfg(test)]
            discard_fault: Default::default(),
        })
    }
}
