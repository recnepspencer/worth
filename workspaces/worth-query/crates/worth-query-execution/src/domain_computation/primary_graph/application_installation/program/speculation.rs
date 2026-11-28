use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_bridge::facade::{
    BridgePreviewBranchBinding, BridgePreviewSessionHandle, BridgePreviewSessionIdentity,
    BridgeSignalBranchIdentity, FinishedBridgePreviewSession, TruthBranchIdentity,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;

mod runtime_custody;
use runtime_custody::PreviewRuntimeDestructionCustody;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationPreviewRequest {
    identity: String,
}

pub(crate) mod discard_fault;

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
}

/// Owns the original native preview, source observation and installed runtime.
/// Dropping this owner abandons it; only explicit successful discard closes it.
/// The original production carriers and both preview phases preserve their
/// structural Send/Sync bounds. Execution still obeys the native owner's affinity.
/// ```
/// use worth_query_execution::facade::application_installation::{
///     WorthQueryApplicationPreviewSession, WorthQueryReadmittedApplicationPreview,
/// };
/// use worth_runtime_bridge::facade::{BridgePreviewSessionHandle, RelationalBridgeObservationLease};
/// fn send_sync<T: Send + Sync>() {}
/// send_sync::<BridgePreviewSessionHandle>();
/// send_sync::<RelationalBridgeObservationLease>();
/// send_sync::<WorthQueryApplicationPreviewSession>();
/// send_sync::<WorthQueryReadmittedApplicationPreview>();
/// ```
pub struct WorthQueryApplicationPreviewSession {
    handle: BridgePreviewSessionHandle,
    _source: std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease>,
    // One actual installed runtime clone, held solely for destruction custody.
    // Native handle and source above are destroyed before this backing is released.
    _runtime: Box<dyn PreviewRuntimeDestructionCustody>,
    runtime_authority: u64,
    source_commit: CompositeCommitIdentity,
    discard_fault: std::sync::Arc<std::sync::atomic::AtomicBool>,
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
            .execution_request_for(runtime.runtime().inner.runtime.product_runtime_ref())
            .map_err(|_| WorthQueryApplicationPreviewReadmissionDenial::ForeignPhase)?;
        if runtime
            .runtime()
            .inner
            .runtime
            .authority_identity()
            .as_u64()
            != self.runtime_authority
        {
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
    pub fn discard(self) -> Result<(), (Self, WorthQueryApplicationPreviewDiscardDenial)> {
        // The court fault refuses before transferring the live Bridge handle.
        if self
            .discard_fault
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err((
                self,
                WorthQueryApplicationPreviewDiscardDenial::CleanupRefused,
            ));
        }
        self.handle.discard();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorthQueryApplicationPreviewDiscardDenial {
    CleanupRefused,
}

impl WorthQueryReadmittedApplicationPreview {
    /// Finishes the admitted preview evidence for the caller.
    pub fn promote(self) -> FinishedBridgePreviewSession {
        self.session.handle.finish()
    }

    pub fn discard(self) -> Result<(), (Self, WorthQueryApplicationPreviewDiscardDenial)> {
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
            .execution_request_for(self.runtime().inner.runtime.product_runtime_ref())
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
        let relational = selected.product().relational_basis().identity().branch_id();
        let signal = selected.product().signal_basis().branch_id();
        let handle = self
            .runtime()
            .inner
            .runtime
            .bridge_ref()
            .ordinary()
            .begin_preview(
                BridgePreviewSessionIdentity::from_stable_name(request.identity),
                BridgePreviewBranchBinding::new(
                    TruthBranchIdentity::from_relational_branch_id(relational.0.clone()),
                    BridgeSignalBranchIdentity::from_stable_name(signal.0.to_string()),
                ),
                source.snapshot_identity().clone(),
            );
        Ok(WorthQueryApplicationPreviewSession {
            handle,
            _source: source,
            _runtime: Box::new(self.clone()),
            runtime_authority: self.runtime().inner.runtime.authority_identity().as_u64(),
            source_commit: observation.selected_commit().clone(),
            discard_fault: Default::default(),
        })
    }
}

mod court_doors_for_test;
