use crate::application::{
    WorthQueryAdmittedDeclarationProgression, WorthQueryDeclarationEntryOrchestrationStage,
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::binding_pipeline::WorthQueryBindingLinkedArtifacts;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::composition::WorthQueryContributionComposedStop;
use super::input::{
    WorthQueryContributionComposedMaterializationPolicy, WorthQueryContributionIntent,
};
use super::intent_result::{
    WorthQueryContributionComposedIntentRequestDescriptor,
    WorthQueryContributionComposedIntentResult,
};
use super::lower::{
    build_composed_artifact, lower_progressed_declaration, process_contributions, stop_reason,
};
use super::mapping::{composed_outcome, linked_artifacts_for_envelope};
use super::outcome::{
    WorthQueryContributionComposedOrchestrationChecked,
    WorthQueryContributionComposedOrchestrationCheckedKind,
};

pub(crate) fn orchestrate_progressed_declaration_with_contributions_checked_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    contributions: Vec<WorthQueryContributionIntent>,
    materialization_policy: WorthQueryContributionComposedMaterializationPolicy,
) -> Result<WorthQueryContributionComposedOrchestrationChecked<D, I>, IdentityDerivationDenial> {
    if contributions.is_empty() {
        return Ok(composed_outcome(
            WorthQueryContributionComposedOrchestrationCheckedKind::Unsupported,
            WorthQueryContributionComposedStop::Unsupported,
            WorthQueryDeclarationEntryOrchestrationStage::DeclarationReviewed,
            "contribution-composed orchestration requires at least one contribution intent",
            WorthQueryBindingLinkedArtifacts::new(),
            None,
            None,
            None,
        ));
    }

    let declaration = match lower_progressed_declaration(handle, progressed) {
        Ok(value) => value,
        Err(outcome) => return Ok(outcome),
    };
    let linked = linked_artifacts_for_envelope(&declaration.envelope);
    let intent_results = process_contributions(
        declaration.target.clone(),
        declaration.declaration_aspect_record.clone(),
        contributions,
        materialization_policy,
        linked.clone(),
    )?;
    finalize_checked(declaration.envelope, linked, intent_results)
}

fn finalize_checked<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    envelope: crate::application::WorthQueryDeclarationEnvelope<D, I>,
    linked: WorthQueryBindingLinkedArtifacts,
    intent_results: Vec<WorthQueryContributionComposedIntentResult>,
) -> Result<WorthQueryContributionComposedOrchestrationChecked<D, I>, IdentityDerivationDenial> {
    let declaration_aspect_record =
        super::WorthQueryContributionComposedDeclarationAspectRecord::new(
            envelope.aspect_contract().clone(),
            envelope.aspect_publication().clone(),
        );
    Ok(
        match build_composed_artifact(envelope, intent_results.clone())? {
            Ok(composed) => WorthQueryContributionComposedOrchestrationChecked::Bound(composed),
            Err((stop, contribution_digest)) => {
                let kind = stop_kind(stop);
                composed_outcome(
                    kind,
                    stop,
                    WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed,
                    stop_reason(stop, &intent_results),
                    linked,
                    Some(contribution_digest),
                    Some(declaration_aspect_record),
                    primary_intent_descriptor(&intent_results).cloned(),
                )
            }
        },
    )
}

fn primary_intent_descriptor(
    intent_results: &[WorthQueryContributionComposedIntentResult],
) -> Option<&WorthQueryContributionComposedIntentRequestDescriptor> {
    intent_results
        .iter()
        .find(|value| !value.is_admitted())
        .or_else(|| intent_results.first())
        .map(WorthQueryContributionComposedIntentResult::request)
}

fn stop_kind(
    stop: WorthQueryContributionComposedStop,
) -> WorthQueryContributionComposedOrchestrationCheckedKind {
    match stop {
        WorthQueryContributionComposedStop::Deferred => {
            WorthQueryContributionComposedOrchestrationCheckedKind::Deferred
        }
        WorthQueryContributionComposedStop::DeclarationDenied => {
            WorthQueryContributionComposedOrchestrationCheckedKind::DeclarationDenied
        }
        WorthQueryContributionComposedStop::ContributionDenied => {
            WorthQueryContributionComposedOrchestrationCheckedKind::ContributionDenied
        }
        WorthQueryContributionComposedStop::Stale => {
            WorthQueryContributionComposedOrchestrationCheckedKind::Stale
        }
        WorthQueryContributionComposedStop::RebindRequired => {
            WorthQueryContributionComposedOrchestrationCheckedKind::RebindRequired
        }
        WorthQueryContributionComposedStop::Unsupported => {
            WorthQueryContributionComposedOrchestrationCheckedKind::Unsupported
        }
        WorthQueryContributionComposedStop::Failed => {
            WorthQueryContributionComposedOrchestrationCheckedKind::Failed
        }
    }
}
