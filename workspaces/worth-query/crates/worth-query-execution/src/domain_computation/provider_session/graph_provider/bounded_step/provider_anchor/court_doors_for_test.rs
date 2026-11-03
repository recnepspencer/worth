//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryGraphProviderAnchor {
    #[doc(hidden)]
    pub fn discard_provisional_overlay_for_test(
        &self, execution: worth_execution::ExecutionRequest<'_, '_>, evidence: WorthQueryProvisionalOverlayEvidenceView<'_>) -> Result<(), WorthQueryProvisionalFailure> {
        Self::discard_provisional_overlay(self, execution, evidence)
    }
}

impl WorthQueryGraphProviderAnchor {
    #[doc(hidden)]
    pub fn stage_provisional_overlay_for_test(
        &self, session: WorthQueryProviderSessionView<'_>, program: WorthQueryProvisionalEffectProgramView<'_>, admission: WorthQueryProvisionalOverlayAdmission) -> Result<WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure> {
        Self::stage_provisional_overlay(self, session, program, admission)
    }
}
