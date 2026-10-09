use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport;

use super::semantic_provider_ports::{
    self, WorthQueryErasedDecisionFactProvider, WorthQueryErasedInvariantExecutionProvider,
    WorthQueryErasedProvisionalGraphProvider,
};
use super::{WorthQueryGraphProviderExecution, WorthQueryGraphProviderExecutionStart};
use crate::domain_computation::{
    WorthQueryBoundInvariantExecutionView, WorthQueryDecisionFactAdmission,
    WorthQueryDecisionFactComparisonAdmission, WorthQueryDecisionFactComparisonEvidence,
    WorthQueryDecisionFactEvidence, WorthQueryDecisionFactEvidenceView,
    WorthQueryDecisionFactRequestView, WorthQueryDecisionReadSetFailure,
    WorthQueryGraphParticipationProvider, WorthQueryGraphProviderCall,
    WorthQueryGraphProviderFailure, WorthQueryInvariantExecutionFailure,
    WorthQueryInvariantProviderVerdict, WorthQueryInvariantStateLoadAdmission,
    WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantStateLoadRequestView,
    WorthQueryInvariantVerdictAdmission, WorthQueryProviderExecutionPlanView,
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionLifecycle,
    WorthQueryProviderSessionToken, WorthQueryProviderSessionTokenAdmission,
    WorthQueryProviderSessionView, WorthQueryProvisionalEffectProgramView,
    WorthQueryProvisionalFailure, WorthQueryProvisionalOverlayAdmission,
    WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalOverlayEvidenceView,
};

pub(super) trait WorthQueryErasedGraphParticipationProvider: Send + Sync {
    fn begin(
        &self,
        call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<Box<dyn WorthQueryGraphProviderExecution>, WorthQueryGraphProviderFailure>;
}

pub(super) trait WorthQueryErasedProviderSessionLifecycle: Send + Sync {
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
        crate::domain_computation::WorthQueryProviderTerminalDescription,
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

pub(crate) enum WorthQueryGraphProviderStartInvocation {
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
        crate::domain_computation::WorthQueryProviderTerminalDescription,
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
    pub(super) provider: Arc<dyn WorthQueryErasedGraphParticipationProvider>,
    pub(super) session_lifecycle: Option<Arc<dyn WorthQueryErasedProviderSessionLifecycle>>,
    pub(super) decision_fact_provider: Option<Arc<dyn WorthQueryErasedDecisionFactProvider>>,
    pub(super) provisional_provider: Option<Arc<dyn WorthQueryErasedProvisionalGraphProvider>>,
    pub(super) invariant_provider: Option<Arc<dyn WorthQueryErasedInvariantExecutionProvider>>,
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

    pub(crate) const fn provider_generation(&self) -> u64 {
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

    pub(crate) fn convergence_provider(
        &self,
    ) -> Option<Arc<dyn crate::domain_computation::WorthQueryConvergenceDomainProvider>> {
        self.convergence_provider.as_ref().map(Arc::clone)
    }

    pub(crate) fn supports_session_protocol(&self) -> bool {
        self.session_lifecycle.is_some()
    }

    #[cfg(test)]
    pub(crate) fn retains_convergence_and_phase_seven_through_ten(&self) -> bool {
        self.convergence_provider.is_some()
            && self.session_lifecycle.is_some()
            && self.decision_fact_provider.is_some()
            && self.provisional_provider.is_some()
            && self.invariant_provider.is_some()
    }

    pub(crate) fn observe_decision_fact(
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

    pub(crate) fn compare_decision_fact(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        semantic_provider_ports::compare_decision_fact(
            &self.decision_fact_provider,
            session,
            evidence,
            admission,
        )
    }

    pub(crate) fn stage_provisional_overlay(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        program: WorthQueryProvisionalEffectProgramView<'_>,
        admission: WorthQueryProvisionalOverlayAdmission,
    ) -> Result<WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure> {
        semantic_provider_ports::stage_provisional_overlay(
            &self.provisional_provider,
            session,
            program,
            admission,
        )
    }

    pub(crate) fn discard_provisional_overlay(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        evidence: WorthQueryProvisionalOverlayEvidenceView<'_>,
    ) -> Result<(), WorthQueryProvisionalFailure> {
        semantic_provider_ports::discard_provisional_overlay(
            &self.provisional_provider,
            execution,
            evidence,
        )
    }

    pub(crate) fn load_invariant_state(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        request: WorthQueryInvariantStateLoadRequestView<'_>,
        admission: WorthQueryInvariantStateLoadAdmission,
    ) -> Result<WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantExecutionFailure> {
        semantic_provider_ports::load_invariant_state(
            &self.invariant_provider,
            session,
            request,
            admission,
        )
    }

    pub(crate) fn execute_invariant(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        execution: WorthQueryBoundInvariantExecutionView<'_>,
        admission: WorthQueryInvariantVerdictAdmission,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
        semantic_provider_ports::execute_invariant(
            &self.invariant_provider,
            session,
            execution,
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
        crate::domain_computation::WorthQueryProviderTerminalDescription,
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
