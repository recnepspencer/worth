use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport;

use super::semantic_provider_ports::{self, WorthQueryErasedDecisionFactProvider};
use super::{WorthQueryGraphProviderExecution, WorthQueryGraphProviderExecutionStart};
use crate::domain_computation::{
    WorthQueryDecisionFactAdmission, WorthQueryDecisionFactComparisonAdmission,
    WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionFactEvidence,
    WorthQueryDecisionFactEvidenceView, WorthQueryDecisionFactRequestView,
    WorthQueryDecisionReadSetFailure, WorthQueryGraphParticipationProvider,
    WorthQueryGraphProviderCall, WorthQueryGraphProviderFailure,
    WorthQueryProviderExecutionPlanView, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionLifecycle, WorthQueryProviderSessionToken,
    WorthQueryProviderSessionTokenAdmission, WorthQueryProviderSessionView,
};

pub(super) trait WorthQueryErasedGraphParticipationProvider:
    std::any::Any + Send + Sync
{
    fn begin(
        &self,
        call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<Box<dyn WorthQueryGraphProviderExecution>, WorthQueryGraphProviderFailure>;
}

pub(super) trait WorthQueryErasedProviderSessionLifecycle:
    std::any::Any + Send + Sync
{
    fn readmit(
        &self,
        plan: &WorthQueryProviderExecutionPlanView<'_>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure>;

    fn prepare(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure>;

    fn prepare_staged(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure>;

    fn commit(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderCommitCompletion,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    >;

    fn abort(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    >;
}

pub enum WorthQueryGraphProviderStartInvocation {
    Returned(Result<Box<dyn WorthQueryGraphProviderExecution>, WorthQueryGraphProviderFailure>),
    Panicked,
}

pub(super) struct WorthQueryTypedGraphParticipationProvider<G, P> {
    pub(super) provider: Arc<P>,
    pub(super) _graph: PhantomData<fn() -> G>,
}

impl<G: 'static, P: WorthQueryGraphParticipationProvider<G>>
    WorthQueryErasedGraphParticipationProvider for WorthQueryTypedGraphParticipationProvider<G, P>
{
    fn begin(
        &self,
        call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<Box<dyn WorthQueryGraphProviderExecution>, WorthQueryGraphProviderFailure> {
        self.provider
            .begin(call, start)
            .and_then(|admitted| start.validate_returned_execution(admitted))
    }
}

impl<P: WorthQueryProviderSessionLifecycle> WorthQueryErasedProviderSessionLifecycle for P {
    fn readmit(
        &self,
        plan: &WorthQueryProviderExecutionPlanView<'_>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure> {
        self.readmit_provider_plan(plan, admission)
    }

    fn prepare(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        self.prepare_provider_session(session)
    }

    fn prepare_staged(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        self.prepare_staged_session(session)
    }

    fn commit(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderCommitCompletion,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        self.commit_prepared_session(session, allocation_policy)
    }

    fn abort(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        self.abort_provider_session(session)
    }
}

pub struct WorthQueryGraphProviderAnchor {
    pub(crate) session_grant_control: Arc<crate::domain_computation::ProviderSessionGrantControl>,
    pub(super) provider: Arc<dyn WorthQueryErasedGraphParticipationProvider>,
    pub(super) session_lifecycle: Option<Arc<dyn WorthQueryErasedProviderSessionLifecycle>>,
    pub(super) decision_fact_provider: Option<Arc<dyn WorthQueryErasedDecisionFactProvider>>,
    pub(super) convergence_provider:
        Option<Arc<dyn crate::domain_computation::WorthQueryConvergenceDomainProvider>>,
    pub(super) provider_identity: &'static str,
    pub(super) provider_generation: u64,
    pub(super) resource_support: WorthQueryExecutionResourceSupport,
}

impl WorthQueryGraphProviderAnchor {
    #[doc(hidden)]
    pub fn provider_identity(&self) -> &'static str {
        self.provider_identity
    }

    pub const fn provider_generation(&self) -> u64 {
        self.provider_generation
    }

    #[doc(hidden)]
    pub fn resource_support(&self) -> &WorthQueryExecutionResourceSupport {
        &self.resource_support
    }

    pub(crate) fn begin(
        &self,
        call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> WorthQueryGraphProviderStartInvocation {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.provider.begin(call, start)
        })) {
            Ok(result) => WorthQueryGraphProviderStartInvocation::Returned(result),
            Err(_) => WorthQueryGraphProviderStartInvocation::Panicked,
        }
    }

    pub fn convergence_provider(
        &self,
    ) -> Option<Arc<dyn crate::domain_computation::WorthQueryConvergenceDomainProvider>> {
        self.convergence_provider.as_ref().map(Arc::clone)
    }

    pub(crate) fn supports_session_protocol(&self) -> bool {
        self.session_lifecycle.is_some()
    }

    pub fn observe_decision_fact(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        request: WorthQueryDecisionFactRequestView<'_>,
        admission: WorthQueryDecisionFactAdmission,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure> {
        semantic_provider_ports::observe_decision_fact(
            &self.decision_fact_provider,
            session,
            request,
            admission,
        )
    }

    pub fn compare_decision_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        semantic_provider_ports::compare_decision_fact(
            execution,
            &self.decision_fact_provider,
            session,
            evidence,
            admission,
        )
    }

    pub(crate) fn readmit_session(
        &self,
        plan: &WorthQueryProviderExecutionPlanView<'_>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure> {
        self.session_lifecycle
            .as_ref()
            .ok_or_else(WorthQueryProviderSessionFailure::unsupported)?
            .readmit(plan, admission)
    }

    pub(crate) fn prepare_session(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        self.session_lifecycle
            .as_ref()
            .ok_or_else(WorthQueryProviderSessionFailure::unsupported)?
            .prepare(session)
    }

    pub(crate) fn prepare_staged_session(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        self.session_lifecycle
            .as_ref()
            .ok_or_else(WorthQueryProviderSessionFailure::unsupported)?
            .prepare_staged(session)
    }

    pub(crate) fn commit_session(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderCommitCompletion,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        let lifecycle = self
            .session_lifecycle
            .as_ref()
            .ok_or_else(WorthQueryProviderSessionFailure::unsupported)
            .map_err(crate::domain_computation::WorthQueryProviderSessionCommitStop::from)?;
        lifecycle.commit(session, allocation_policy)
    }

    pub(crate) fn abort_session(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        self.session_lifecycle
            .as_ref()
            .ok_or_else(WorthQueryProviderSessionFailure::unsupported)?
            .abort(session)
    }
}

/// Crate-boundary internal port for `WorthQueryGraphProviderAnchor` session
/// grant control. Never exported on `worth-query-execution::facade` or Host.
pub trait GraphProviderAnchorInternalPort {
    fn session_grant_control(&self)
        -> &Arc<crate::domain_computation::ProviderSessionGrantControl>;
}

impl GraphProviderAnchorInternalPort for WorthQueryGraphProviderAnchor {
    fn session_grant_control(
        &self,
    ) -> &Arc<crate::domain_computation::ProviderSessionGrantControl> {
        &self.session_grant_control
    }
}
