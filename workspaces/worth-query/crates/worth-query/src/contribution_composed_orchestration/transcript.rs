use crate::application::{
    WorthQueryDeclarationEntryOrchestrationStage, WorthQueryDeclarationInput,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::binding_pipeline::WorthQueryBindingLinkedArtifacts;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::composition::WorthQueryContributionComposedClassification;
use super::declaration_record::WorthQueryContributionComposedDeclarationRecord;
use super::input::WorthQueryContributionComposedOrchestrationInput;
use super::intent_result::{primary_intent_descriptor, WorthQueryContributionComposedIntentResult};
use super::lower::{
    build_composed_artifact, lower_declaration, materialization_policy_label,
    process_contributions, request_descriptor, request_identity, stop_reason, DeclarationLowering,
};
use super::mapping::{
    composed_outcome, contribution_digest_from_outcome, linked_artifacts_for_envelope,
    linked_artifacts_from_outcome,
};
use super::outcome::{
    WorthQueryContributionComposedOrchestrationChecked,
    WorthQueryContributionComposedOrchestrationCheckedKind,
    WorthQueryContributionComposedOrchestrationOutcome,
};

pub struct WorthQueryContributionComposedOrchestrationTranscript<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    request_descriptor: String,
    request_digest: String,
    materialization_policy: &'static str,
    declaration: WorthQueryContributionComposedDeclarationRecord,
    outcome: WorthQueryContributionComposedOrchestrationOutcome<D, I>,
    linked_artifacts: WorthQueryBindingLinkedArtifacts,
    contribution_digest: Option<String>,
    composition_classification: Option<WorthQueryContributionComposedClassification>,
    intent_results: Vec<WorthQueryContributionComposedIntentResult>,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryContributionComposedOrchestrationTranscript<D, I>
{
    pub(crate) fn new(
        request_descriptor: String,
        request_digest: String,
        materialization_policy: &'static str,
        declaration: WorthQueryContributionComposedDeclarationRecord,
        outcome: WorthQueryContributionComposedOrchestrationOutcome<D, I>,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
        contribution_digest: Option<String>,
        composition_classification: Option<WorthQueryContributionComposedClassification>,
        intent_results: Vec<WorthQueryContributionComposedIntentResult>,
    ) -> Self {
        Self {
            request_descriptor,
            request_digest,
            materialization_policy,
            declaration,
            outcome,
            linked_artifacts,
            contribution_digest,
            composition_classification,
            intent_results,
        }
    }

    pub fn request_descriptor(&self) -> &str {
        &self.request_descriptor
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }

    pub fn materialization_policy(&self) -> &str {
        self.materialization_policy
    }

    pub fn declaration(&self) -> &WorthQueryContributionComposedDeclarationRecord {
        &self.declaration
    }

    pub fn outcome(&self) -> &WorthQueryContributionComposedOrchestrationOutcome<D, I> {
        &self.outcome
    }

    pub fn linked_artifacts(&self) -> &WorthQueryBindingLinkedArtifacts {
        &self.linked_artifacts
    }

    pub fn contribution_digest(&self) -> Option<&str> {
        self.contribution_digest.as_deref()
    }

    pub fn composition_classification(
        &self,
    ) -> Option<WorthQueryContributionComposedClassification> {
        self.composition_classification
    }

    pub fn intent_results(&self) -> &[WorthQueryContributionComposedIntentResult] {
        &self.intent_results
    }

    pub fn into_checked(self) -> WorthQueryContributionComposedOrchestrationChecked<D, I> {
        self.outcome
    }
}

pub(crate) fn orchestrate_declaration_with_contributions_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    input: WorthQueryContributionComposedOrchestrationInput<D, I>,
) -> Result<WorthQueryContributionComposedOrchestrationTranscript<D, I>, IdentityDerivationDenial> {
    let request_descriptor_value = request_descriptor(&input);
    let request_digest_value = request_identity(&input)?.as_str().to_string();
    let materialization_policy = materialization_policy_label(input.materialization_policy());
    let (declaration_input, contributions, materialization_policy_value) = input.into_parts();
    if contributions.is_empty() {
        return Ok(empty_contribution_transcript(
            request_descriptor_value,
            request_digest_value,
            materialization_policy,
        ));
    }
    let declaration = match lower_declaration(handle, declaration_input) {
        Ok(value) => value,
        Err(outcome) => {
            let linked = linked_artifacts_from_outcome(&outcome)?;
            let contribution_digest = contribution_digest_from_outcome(&outcome)?;
            return Ok(WorthQueryContributionComposedOrchestrationTranscript::new(
                request_descriptor_value,
                request_digest_value,
                materialization_policy,
                WorthQueryContributionComposedDeclarationRecord::from_linked_artifacts(
                    declaration_stage(&outcome)?,
                    &linked,
                ),
                outcome,
                linked.clone(),
                contribution_digest,
                None,
                Vec::new(),
            ));
        }
    };
    let linked = linked_artifacts_for_envelope(&declaration.envelope);
    let intent_results = process_contributions(
        declaration.target.clone(),
        declaration.declaration_aspect_record.clone(),
        contributions,
        materialization_policy_value,
        linked.clone(),
    )?;
    Ok(finalize_transcript(
        request_descriptor_value,
        request_digest_value,
        materialization_policy,
        declaration,
        linked,
        intent_results,
    )?)
}

fn finalize_transcript<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    request_descriptor: String,
    request_digest: String,
    materialization_policy: &'static str,
    declaration: DeclarationLowering<D, I>,
    linked: WorthQueryBindingLinkedArtifacts,
    intent_results: Vec<WorthQueryContributionComposedIntentResult>,
) -> Result<WorthQueryContributionComposedOrchestrationTranscript<D, I>, IdentityDerivationDenial> {
    let declaration_record = WorthQueryContributionComposedDeclarationRecord::from_envelope(
        &declaration.envelope,
        &linked,
    );
    Ok(
        match build_composed_artifact(declaration.envelope, intent_results.clone())? {
            Ok(composed) => {
                let composition_classification = Some(composed.classification());
                let contribution_digest = Some(composed.composition_for_reporting().to_string());
                WorthQueryContributionComposedOrchestrationTranscript::new(
                    request_descriptor,
                    request_digest,
                    materialization_policy,
                    declaration_record,
                    WorthQueryContributionComposedOrchestrationOutcome::Bound(composed),
                    linked,
                    contribution_digest,
                    composition_classification,
                    intent_results,
                )
            }
            Err((stop, contribution_digest)) => {
                let (kind, reason) = stop_outcome_kind(stop, &intent_results);
                let outcome = composed_outcome(
                    kind,
                    stop,
                    WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed,
                    reason,
                    linked.clone(),
                    Some(contribution_digest.clone()),
                    declaration_record.aspect_record().cloned(),
                    primary_intent_descriptor(&intent_results).cloned(),
                );
                WorthQueryContributionComposedOrchestrationTranscript::new(
                    request_descriptor,
                    request_digest,
                    materialization_policy,
                    declaration_record,
                    outcome,
                    linked,
                    Some(contribution_digest),
                    None,
                    intent_results,
                )
            }
        },
    )
}

fn empty_contribution_transcript<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    request_descriptor: String,
    request_digest: String,
    materialization_policy: &'static str,
) -> WorthQueryContributionComposedOrchestrationTranscript<D, I> {
    let outcome = super::mapping::composed_outcome(
        WorthQueryContributionComposedOrchestrationCheckedKind::Unsupported,
        super::composition::WorthQueryContributionComposedStop::Unsupported,
        WorthQueryDeclarationEntryOrchestrationStage::DeclarationReviewed,
        "contribution-composed orchestration requires at least one contribution intent",
        WorthQueryBindingLinkedArtifacts::new(),
        None,
        None,
        None,
    );
    WorthQueryContributionComposedOrchestrationTranscript::new(
        request_descriptor,
        request_digest,
        materialization_policy,
        WorthQueryContributionComposedDeclarationRecord::from_linked_artifacts(
            WorthQueryDeclarationEntryOrchestrationStage::DeclarationReviewed,
            &WorthQueryBindingLinkedArtifacts::new(),
        ),
        outcome,
        WorthQueryBindingLinkedArtifacts::new(),
        None,
        None,
        Vec::new(),
    )
}

fn declaration_stage<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    outcome: &WorthQueryContributionComposedOrchestrationOutcome<D, I>,
) -> Result<WorthQueryDeclarationEntryOrchestrationStage, IdentityDerivationDenial> {
    Ok(match outcome {
        WorthQueryContributionComposedOrchestrationOutcome::EvidenceIdentityDerivationDenied(
            cause,
        ) => return Err(*cause),

        WorthQueryContributionComposedOrchestrationOutcome::Bound(_) => {
            WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed
        }
        WorthQueryContributionComposedOrchestrationOutcome::Deferred(value)
        | WorthQueryContributionComposedOrchestrationOutcome::DeclarationDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::ContributionDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Stale(value)
        | WorthQueryContributionComposedOrchestrationOutcome::RebindRequired(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Unsupported(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Failed(value) => value.stop_stage(),
    })
}

fn stop_outcome_kind(
    stop: super::composition::WorthQueryContributionComposedStop,
    intent_results: &[WorthQueryContributionComposedIntentResult],
) -> (
    WorthQueryContributionComposedOrchestrationCheckedKind,
    String,
) {
    let kind = match stop {
        super::composition::WorthQueryContributionComposedStop::Deferred => {
            WorthQueryContributionComposedOrchestrationCheckedKind::Deferred
        }
        super::composition::WorthQueryContributionComposedStop::DeclarationDenied => {
            WorthQueryContributionComposedOrchestrationCheckedKind::DeclarationDenied
        }
        super::composition::WorthQueryContributionComposedStop::ContributionDenied => {
            WorthQueryContributionComposedOrchestrationCheckedKind::ContributionDenied
        }
        super::composition::WorthQueryContributionComposedStop::Stale => {
            WorthQueryContributionComposedOrchestrationCheckedKind::Stale
        }
        super::composition::WorthQueryContributionComposedStop::RebindRequired => {
            WorthQueryContributionComposedOrchestrationCheckedKind::RebindRequired
        }
        super::composition::WorthQueryContributionComposedStop::Unsupported => {
            WorthQueryContributionComposedOrchestrationCheckedKind::Unsupported
        }
        super::composition::WorthQueryContributionComposedStop::Failed => {
            WorthQueryContributionComposedOrchestrationCheckedKind::Failed
        }
    };
    (kind, stop_reason(stop, intent_results))
}
