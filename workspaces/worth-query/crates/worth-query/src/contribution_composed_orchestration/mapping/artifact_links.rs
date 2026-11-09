use super::super::outcome::WorthQueryContributionComposedOrchestrationOutcome;
use crate::application::{
    WorthQueryDeclarationEnvelope, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
};
use crate::binding_pipeline::WorthQueryBindingLinkedArtifacts;

pub(in crate::contribution_composed_orchestration) fn linked_artifacts_for_envelope<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
) -> WorthQueryBindingLinkedArtifacts {
    let mut linked = WorthQueryBindingLinkedArtifacts::new()
        .with_declaration_digest(envelope.declaration_digest().to_string());
    if let Some(value) = envelope.progression_digest() {
        linked = linked.with_progression_digest(value.to_string());
    }
    if let Some(value) = envelope.route_plan_digest() {
        linked = linked.with_route_plan_digest(value.to_string());
    }
    linked
        .with_receipt_digest(format!("{:?}", envelope.receipt_digest()))
        .with_envelope_digest(format!("{:?}", envelope.envelope_digest()))
}

pub(in crate::contribution_composed_orchestration) fn linked_artifacts_from_outcome<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    outcome: &WorthQueryContributionComposedOrchestrationOutcome<D, I>,
) -> Result<
    WorthQueryBindingLinkedArtifacts,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(match outcome {
        WorthQueryContributionComposedOrchestrationOutcome::EvidenceIdentityDerivationDenied(
            cause,
        ) => return Err(*cause),
        WorthQueryContributionComposedOrchestrationOutcome::Bound(value) => {
            linked_artifacts_for_envelope(value.envelope())
        }
        WorthQueryContributionComposedOrchestrationOutcome::Deferred(value)
        | WorthQueryContributionComposedOrchestrationOutcome::DeclarationDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::ContributionDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Stale(value)
        | WorthQueryContributionComposedOrchestrationOutcome::RebindRequired(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Unsupported(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Failed(value) => {
            value.linked_artifacts().clone()
        }
    })
}

pub(in crate::contribution_composed_orchestration) fn contribution_digest_from_outcome<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    outcome: &WorthQueryContributionComposedOrchestrationOutcome<D, I>,
) -> Result<Option<String>, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(match outcome {
        WorthQueryContributionComposedOrchestrationOutcome::EvidenceIdentityDerivationDenied(
            cause,
        ) => return Err(*cause),
        WorthQueryContributionComposedOrchestrationOutcome::Bound(value) => {
            Some(value.composition_for_reporting().to_string())
        }
        WorthQueryContributionComposedOrchestrationOutcome::Deferred(value)
        | WorthQueryContributionComposedOrchestrationOutcome::DeclarationDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::ContributionDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Stale(value)
        | WorthQueryContributionComposedOrchestrationOutcome::RebindRequired(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Unsupported(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Failed(value) => {
            value.contribution_digest().map(str::to_string)
        }
    })
}
