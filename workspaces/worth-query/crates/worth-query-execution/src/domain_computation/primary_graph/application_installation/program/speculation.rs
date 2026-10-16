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
}

/// Owns the original native preview, source observation and installed runtime.
/// Dropping this owner abandons it; only explicit successful discard closes it.
pub struct WorthQueryApplicationPreviewSession {
    handle: BridgeSpeculativeSessionHandle,
    _source: std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease>,
    // The actual installed runtime clone, erased only for destruction custody.
    // Native handle and source above are destroyed before this backing is released.
    _runtime: Box<dyn std::any::Any>,
    runtime_authority: u64,
    source_commit: CompositeCommitIdentity,
    #[cfg(test)]
    discard_fault: std::rc::Rc<std::cell::Cell<bool>>,
}

impl std::fmt::Debug for WorthQueryApplicationPreviewSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationPreviewSession")
            .field("handle", &self.handle)
            .field("source_commit", &self.source_commit)
            .finish_non_exhaustive()
    }
}

/// Original preview evidence admitted against its exact source at readmission.
/// Promotion finalizes that evidence; it does not certify current Query output.
#[derive(Debug)]
pub struct WorthQueryReadmittedApplicationPreview {
    session: WorthQueryApplicationPreviewSession,
}

impl WorthQueryApplicationPreviewSession {
    /// Fresh selection refusal returns the complete original owner without cleanup.
    pub fn readmit<Schema, Program>(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryReadmittedApplicationPreview,
        (Self, WorthQueryApplicationPreviewReadmissionDenial),
    >
    where
        Schema: ApplicationSchema,
        Program: ApplicationProgramDefinition<Schema>,
    {
        match self.check_readmission(phase, runtime) {
            Ok(()) => Ok(WorthQueryReadmittedApplicationPreview { session: self }),
            Err(denial) => Err((self, denial)),
        }
    }

    fn check_readmission<Schema, Program>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<(), WorthQueryApplicationPreviewReadmissionDenial>
    where
        Schema: ApplicationSchema,
        Program: ApplicationProgramDefinition<Schema>,
    {
        let execution = phase
            .execution_request_for(&runtime.runtime().product_runtime)
            .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::ForeignPhase)?;
        if runtime.runtime().runtime.authority_identity().as_u64() != self.runtime_authority {
            return Err(WorthQueryApplicationPreviewReadmissionDenial::ForeignRuntime);
        }
        let current = runtime
            .on_branch(runtime.current_world())
            .select_with_request(&execution)
            .map_err(WorthQueryApplicationPreviewReadmissionDenial::SourceSelection)?;
        if current.product().selected_commit() != &self.source_commit {
            return Err(WorthQueryApplicationPreviewReadmissionDenial::StaleSource);
        }
        Ok(())
    }

    /// Refusal returns the same preview, including its source observation.
    pub fn discard(
        mut self,
    ) -> Result<(), (Self, worth_runtime_bridge::facade::BridgeSpeculationError)> {
        let residue = vec![BridgePreviewResidueClass::TemporaryDiagnosticsResidue];
        #[cfg(test)]
        let residue = if self.discard_fault.get() {
            vec![BridgePreviewResidueClass::AuthoritativeDiagnosticsResidue]
        } else {
            residue
        };
        match self.handle.discard(residue) {
            Ok(_) => Ok(()),
            Err((handle, denial)) => {
                self.handle = handle;
                Err((self, denial))
            }
        }
    }
}

impl WorthQueryReadmittedApplicationPreview {
    /// Finalizes the original admitted preview evidence, not current Query output.
    pub fn promote(
        mut self,
    ) -> Result<
        BridgeSpeculativePromotionOutcome,
        (Self, worth_runtime_bridge::facade::BridgeSpeculationError),
    > {
        match self.session.handle.promote() {
            Ok(outcome) => Ok(outcome),
            Err((handle, denial)) => {
                self.session.handle = handle;
                Err((self, denial))
            }
        }
    }

    pub fn discard(
        self,
    ) -> Result<(), (Self, worth_runtime_bridge::facade::BridgeSpeculationError)> {
        self.session
            .discard()
            .map_err(|(session, denial)| (Self { session }, denial))
    }
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
            handle,
            _source: source,
            _runtime: Box::new(self.clone()),
            runtime_authority: self.runtime().runtime.authority_identity().as_u64(),
            source_commit: observation.selected_commit().clone(),
            #[cfg(test)]
            discard_fault: Default::default(),
        })
    }
}
