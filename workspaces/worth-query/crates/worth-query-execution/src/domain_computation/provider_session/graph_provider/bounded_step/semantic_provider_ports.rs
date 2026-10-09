use std::sync::Arc;

use crate::domain_computation::{
    WorthQueryBoundInvariantExecutionView, WorthQueryDecisionFactAdmission,
    WorthQueryDecisionFactComparisonAdmission, WorthQueryDecisionFactComparisonEvidence,
    WorthQueryDecisionFactEvidence, WorthQueryDecisionFactEvidenceView,
    WorthQueryDecisionFactProvider, WorthQueryDecisionFactRequestView,
    WorthQueryDecisionReadSetDenialKind, WorthQueryDecisionReadSetFailure,
    WorthQueryInvariantExecutionFailure, WorthQueryInvariantExecutionProvider,
    WorthQueryInvariantProviderVerdict, WorthQueryInvariantStateLoadAdmission,
    WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantStateLoadRequestView,
    WorthQueryInvariantVerdictAdmission, WorthQueryProvisionalDenialKind,
    WorthQueryProvisionalEffectProgramView, WorthQueryProvisionalFailure,
    WorthQueryProvisionalGraphProvider, WorthQueryProvisionalOverlayAdmission,
    WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalOverlayEvidenceView,
};

pub(super) trait WorthQueryErasedDecisionFactProvider: Send + Sync {
    fn observe(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        request: WorthQueryDecisionFactRequestView<'_>,
        admission: WorthQueryDecisionFactAdmission,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure>;

    fn compare(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure>;
}

pub(super) trait WorthQueryErasedProvisionalGraphProvider: Send + Sync {
    fn stage(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        program: WorthQueryProvisionalEffectProgramView<'_>,
        admission: WorthQueryProvisionalOverlayAdmission,
    ) -> Result<WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure>;

    fn discard(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        evidence: WorthQueryProvisionalOverlayEvidenceView<'_>,
    ) -> Result<(), WorthQueryProvisionalFailure>;
}

pub(super) trait WorthQueryErasedInvariantExecutionProvider: Send + Sync {
    fn load(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        request: WorthQueryInvariantStateLoadRequestView<'_>,
        admission: WorthQueryInvariantStateLoadAdmission,
    ) -> Result<WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantExecutionFailure>;

    fn execute(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        execution: WorthQueryBoundInvariantExecutionView<'_>,
        admission: WorthQueryInvariantVerdictAdmission,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure>;
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
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        self.compare_decision_fact(session, evidence, admission)
    }
}

impl<P: WorthQueryProvisionalGraphProvider> WorthQueryErasedProvisionalGraphProvider for P {
    fn stage(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        program: WorthQueryProvisionalEffectProgramView<'_>,
        admission: WorthQueryProvisionalOverlayAdmission,
    ) -> Result<WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure> {
        self.stage_provisional_overlay(session, program, admission)
    }

    fn discard(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        evidence: WorthQueryProvisionalOverlayEvidenceView<'_>,
    ) -> Result<(), WorthQueryProvisionalFailure> {
        self.discard_provisional_overlay(execution, evidence)
    }
}

impl<P: WorthQueryInvariantExecutionProvider> WorthQueryErasedInvariantExecutionProvider for P {
    fn load(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        request: WorthQueryInvariantStateLoadRequestView<'_>,
        admission: WorthQueryInvariantStateLoadAdmission,
    ) -> Result<WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantExecutionFailure> {
        self.load_invariant_state(session, request, admission)
    }

    fn execute(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        execution: WorthQueryBoundInvariantExecutionView<'_>,
        admission: WorthQueryInvariantVerdictAdmission,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
        self.execute_invariant(session, execution, admission)
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
        .compare(session, evidence, admission)
}

pub(super) fn stage_provisional_overlay(
    provider: &Option<Arc<dyn WorthQueryErasedProvisionalGraphProvider>>,
    session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    program: WorthQueryProvisionalEffectProgramView<'_>,
    admission: WorthQueryProvisionalOverlayAdmission,
) -> Result<WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure> {
    let provider = provider.as_ref().ok_or_else(|| {
        WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::ProviderUnsupported,
            "provider did not install provisional overlay authority",
        )
    })?;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        provider.stage(session, program, admission)
    }))
    .unwrap_or_else(|_| {
        Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::ProviderPanicked,
            "provider panicked while staging provisional overlay",
        ))
    })
}

pub(super) fn discard_provisional_overlay(
    provider: &Option<Arc<dyn WorthQueryErasedProvisionalGraphProvider>>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    evidence: WorthQueryProvisionalOverlayEvidenceView<'_>,
) -> Result<(), WorthQueryProvisionalFailure> {
    let provider = provider.as_ref().ok_or_else(|| {
        WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::ProviderUnsupported,
            "provider did not install provisional overlay authority",
        )
    })?;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        provider.discard(execution, evidence)
    }))
    .unwrap_or_else(|_| {
        Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::ProviderPanicked,
            "provider panicked while discarding provisional overlay",
        ))
    })
}

pub(super) fn load_invariant_state(
    provider: &Option<Arc<dyn WorthQueryErasedInvariantExecutionProvider>>,
    session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    request: WorthQueryInvariantStateLoadRequestView<'_>,
    admission: WorthQueryInvariantStateLoadAdmission,
) -> Result<WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantExecutionFailure> {
    let provider = invariant_provider(provider)?;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        provider.load(session, request, admission)
    }))
    .unwrap_or_else(|_| {
        Err(WorthQueryInvariantExecutionFailure::new(
            crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderPanicked,
            "provider panicked while loading invariant state",
        ))
    })
}

pub(super) fn execute_invariant(
    provider: &Option<Arc<dyn WorthQueryErasedInvariantExecutionProvider>>,
    session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    execution: WorthQueryBoundInvariantExecutionView<'_>,
    admission: WorthQueryInvariantVerdictAdmission,
) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
    let provider = invariant_provider(provider)?;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        provider.execute(session, execution, admission)
    }))
    .unwrap_or_else(|_| {
        Err(WorthQueryInvariantExecutionFailure::new(
            crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderPanicked,
            "provider panicked while executing invariant",
        ))
    })
}

fn invariant_provider(
    provider: &Option<Arc<dyn WorthQueryErasedInvariantExecutionProvider>>,
) -> Result<&Arc<dyn WorthQueryErasedInvariantExecutionProvider>, WorthQueryInvariantExecutionFailure>
{
    provider.as_ref().ok_or_else(|| {
        WorthQueryInvariantExecutionFailure::new(
            crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderUnsupported,
            "provider did not install invariant execution authority",
        )
    })
}
