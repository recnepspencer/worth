use std::sync::Arc;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryProposedFact, WorthQueryProposedFactOrigin, WorthQueryProviderSessionView,
    WorthQueryProvisionalEffectAction, WorthQueryProvisionalEffectProgramView,
    WorthQueryProvisionalFailure, WorthQueryProvisionalGraphProvider,
    WorthQueryProvisionalOverlayAdmission, WorthQueryProvisionalOverlayEvidence,
    WorthQueryProvisionalOverlayEvidenceView,
};

impl WorthQueryProvisionalGraphProvider for Arc<WorthQueryPrimaryGraphProvider> {
    fn stage_provisional_overlay(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        program: WorthQueryProvisionalEffectProgramView<'_>,
        admission: WorthQueryProvisionalOverlayAdmission,
    ) -> Result<WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure> {
        let facts = program
            .steps()
            .iter()
            .map(|step| proposed_fact(step.action()))
            .collect::<Result<Vec<_>, _>>()?;
        self.application_attempt_work.observe_overlay_staging();
        let overlay = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .stage_overlay(session, program.steps(), program.generation(), facts)
            .map_err(WorthQueryProvisionalFailure::invalid_program)?;
        overlay.admit(admission)
    }

    fn discard_provisional_overlay(
        &self,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
        evidence: WorthQueryProvisionalOverlayEvidenceView<'_>,
    ) -> Result<(), WorthQueryProvisionalFailure> {
        let discarded = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .discard_overlay(evidence);
        discarded.then_some(()).ok_or_else(|| {
            WorthQueryProvisionalFailure::invalid_program(
                "provider overlay evidence does not belong to the exact application attempt",
            )
        })
    }
}

fn proposed_fact(
    action: &WorthQueryProvisionalEffectAction,
) -> Result<WorthQueryProposedFact, WorthQueryProvisionalFailure> {
    let (identity, origin, value) = match action {
        WorthQueryProvisionalEffectAction::Create { symbolic_identity } => (
            symbolic_identity.as_ref(),
            WorthQueryProposedFactOrigin::StagedCreation,
            "created",
        ),
        WorthQueryProvisionalEffectAction::Replace { target_identity } => (
            target_identity.as_ref(),
            WorthQueryProposedFactOrigin::StagedReplacement,
            "replaced",
        ),
        WorthQueryProvisionalEffectAction::Retire { target_identity } => (
            target_identity.as_ref(),
            WorthQueryProposedFactOrigin::StagedRetirement,
            "retired",
        ),
        WorthQueryProvisionalEffectAction::DeriveView { view_identity } => (
            view_identity.as_ref(),
            WorthQueryProposedFactOrigin::DerivedProvisionalView,
            "derived",
        ),
    };
    WorthQueryProposedFact::new(identity, origin, value)
}
