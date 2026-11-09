use crate::application::aspect_coverage_from_publication;
use crate::application::{
    WorthQueryAdmittedDeclarationProgression, WorthQueryDeclarationEnvelope,
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::domain_capabilities::WorthQueryInstalledDeclarationContributionTarget;

use super::super::aspect::WorthQueryContributionComposedIntentAspectRecord;
use super::super::mapping::{declaration_error_outcome, envelope_error_outcome};
use super::super::outcome::WorthQueryContributionComposedOrchestrationOutcome;

pub(crate) struct DeclarationLowering<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    pub(crate) envelope: WorthQueryDeclarationEnvelope<D, I>,
    pub(crate) target: WorthQueryInstalledDeclarationContributionTarget,
    pub(crate) declaration_aspect_record: WorthQueryContributionComposedIntentAspectRecord,
}

pub(crate) fn lower_declaration<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    declaration_input: I,
) -> Result<DeclarationLowering<D, I>, WorthQueryContributionComposedOrchestrationOutcome<D, I>> {
    let progressed = match handle.declare_review_and_progress(declaration_input) {
        Ok(progressed) => progressed,
        Err(error) => return Err(declaration_error_outcome(error)),
    };
    lower_progressed_declaration(handle, progressed)
}

pub(crate) fn lower_progressed_declaration<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<DeclarationLowering<D, I>, WorthQueryContributionComposedOrchestrationOutcome<D, I>> {
    let target = handle.contribution_target(progressed.canonical_declaration())?;
    let envelope = match handle.orchestrate_envelope_from_progressed(progressed) {
        Ok(envelope) => envelope,
        Err(error) => return Err(envelope_error_outcome(error)),
    };
    let declaration_aspect_record = WorthQueryContributionComposedIntentAspectRecord::new(
        envelope.aspect_contract().clone(),
        aspect_coverage_from_publication(envelope.aspect_publication()),
    );
    Ok(DeclarationLowering {
        envelope,
        target,
        declaration_aspect_record,
    })
}
