use std::sync::Arc;

use crate::domain_computation::{
    WorthQueryDecisionFactAdmission, WorthQueryDecisionFactComparisonAdmission,
    WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionFactEvidence,
    WorthQueryDecisionFactEvidenceView, WorthQueryDecisionFactProvider,
    WorthQueryDecisionFactRequestView, WorthQueryDecisionReadSetDenialKind,
    WorthQueryDecisionReadSetFailure,
};

pub(super) trait WorthQueryErasedDecisionFactProvider: std::any::Any + Send + Sync {
    fn observe(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        request: WorthQueryDecisionFactRequestView<'_>,
        admission: WorthQueryDecisionFactAdmission,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure>;

    fn compare(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure>;
}

impl<P: WorthQueryDecisionFactProvider> WorthQueryErasedDecisionFactProvider for P {
    fn observe(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        request: WorthQueryDecisionFactRequestView<'_>,
        admission: WorthQueryDecisionFactAdmission,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure> {
        self.observe_decision_fact(session, request, admission)
    }

    fn compare(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        self.compare_decision_fact(execution, session, evidence, admission)
    }
}

pub(super) fn observe_decision_fact(
    provider: &Option<Arc<dyn WorthQueryErasedDecisionFactProvider>>,
    session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    request: WorthQueryDecisionFactRequestView<'_>,
    admission: WorthQueryDecisionFactAdmission,
) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure> {
    provider
        .as_ref()
        .ok_or_else(|| {
            WorthQueryDecisionReadSetFailure::new(
                WorthQueryDecisionReadSetDenialKind::DecisionFactsUnsupported,
                "provider did not install decision-fact observation authority",
            )
        })?
        .observe(session, request, admission)
}

pub(super) fn compare_decision_fact(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &Option<Arc<dyn WorthQueryErasedDecisionFactProvider>>,
    session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    evidence: WorthQueryDecisionFactEvidenceView<'_>,
    admission: WorthQueryDecisionFactComparisonAdmission,
) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
    provider
        .as_ref()
        .ok_or_else(|| {
            WorthQueryDecisionReadSetFailure::new(
                WorthQueryDecisionReadSetDenialKind::DecisionFactsUnsupported,
                "provider did not install decision-fact comparison authority",
            )
        })?
        .compare(execution, session, evidence, admission)
}
