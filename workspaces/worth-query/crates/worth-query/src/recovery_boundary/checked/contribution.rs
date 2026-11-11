use crate::application::{WorthQueryDeclarationInput, WorthQueryDomainEntryMarker};
use crate::contribution_composed_orchestration::{
    ordinary_outcome_from_contribution_composed_checked,
    WorthQueryContributionComposedClassification,
    WorthQueryContributionComposedIntentClassification,
    WorthQueryContributionComposedIntentRequestDescriptor,
    WorthQueryContributionComposedOrchestrationChecked,
    WorthQueryContributionComposedOrchestrationCheckedKind,
    WorthQueryContributionComposedOrchestrationOutcome,
    WorthQueryContributionComposedOrchestrationTranscript,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::recovery_boundary::foundational::{
    diagnostic_context_for_stop_kind, support_context_for_stale_basis,
};
use crate::recovery_boundary::ordinary::worth_query_recovery_brief_from_ordinary_outcome;
use crate::recovery_boundary::{
    WorthQueryRecoveryAspectPosture, WorthQueryRecoveryBasisPosture, WorthQueryRecoveryBrief,
    WorthQueryRecoveryConflictPosture, WorthQueryRecoveryEvidenceStrength,
    WorthQueryRecoveryExplanation, WorthQueryRecoveryFoundationalSupportContext,
    WorthQueryRecoverySourceFamily, WorthQueryRecoveryStopKind,
};

pub fn worth_query_recovery_brief_from_contribution_composed_checked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    checked: WorthQueryContributionComposedOrchestrationChecked<D, I>,
) -> Result<Option<WorthQueryRecoveryBrief>, IdentityDerivationDenial> {
    let explanation = contribution_checked_explanation(
        &checked,
        WorthQueryRecoveryEvidenceStrength::CheckedRetained,
    )?;
    Ok(worth_query_recovery_brief_from_ordinary_outcome(
        &ordinary_outcome_from_contribution_composed_checked(checked)?,
    )
    .map(|brief| {
        let merged = merge_contribution_explanation(&brief, explanation);
        brief.with_explanation(merged)
    }))
}

pub fn worth_query_recovery_brief_from_contribution_composed_proof<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    proof: WorthQueryContributionComposedOrchestrationTranscript<D, I>,
) -> Result<Option<WorthQueryRecoveryBrief>, IdentityDerivationDenial> {
    let explanation =
        contribution_proof_explanation(&proof, WorthQueryRecoveryEvidenceStrength::ProofRetained);
    Ok(worth_query_recovery_brief_from_ordinary_outcome(
        &ordinary_outcome_from_contribution_composed_checked(proof.into_checked())?,
    )
    .map(|brief| {
        let merged = merge_contribution_explanation(&brief, explanation);
        brief.with_explanation(merged)
    }))
}

fn merge_contribution_explanation(
    brief: &WorthQueryRecoveryBrief,
    explanation: WorthQueryRecoveryExplanation,
) -> WorthQueryRecoveryExplanation {
    let mut merged = brief
        .explanation()
        .clone()
        .with_source_family(explanation.source_family())
        .with_evidence_strength(explanation.evidence_strength())
        .with_basis_posture(explanation.basis_posture())
        .with_aspect_posture(explanation.aspect_posture())
        .with_conflict_posture(explanation.conflict_posture())
        .with_diagnostic_context(diagnostic_context_for_stop_kind(brief.stop_kind()));
    if let (Some(truth_kind), Some(basis_disclosure)) = (
        explanation.support_truth_kind(),
        explanation.basis_disclosure(),
    ) {
        merged = merged.with_support_context(WorthQueryRecoveryFoundationalSupportContext::new(
            truth_kind,
            basis_disclosure,
            explanation.degraded_recovery_posture(),
        ));
    }
    if let Some(descriptor) = explanation.contribution_intent_descriptor() {
        merged = merged.with_contribution_intent_descriptor(descriptor.clone());
    }
    merged
}

fn contribution_checked_explanation<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    checked: &WorthQueryContributionComposedOrchestrationChecked<D, I>,
    evidence_strength: WorthQueryRecoveryEvidenceStrength,
) -> Result<WorthQueryRecoveryExplanation, IdentityDerivationDenial> {
    let (kind, linked) = match checked {
        WorthQueryContributionComposedOrchestrationOutcome::EvidenceIdentityDerivationDenied(
            cause,
        ) => return Err(*cause),

        WorthQueryContributionComposedOrchestrationOutcome::Bound(_) => {
            return Ok(base_contribution_explanation(
                evidence_strength,
                WorthQueryRecoveryBasisPosture::Unknown,
                WorthQueryRecoveryConflictPosture::None,
            ));
        }
        WorthQueryContributionComposedOrchestrationOutcome::Deferred(value)
        | WorthQueryContributionComposedOrchestrationOutcome::DeclarationDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::ContributionDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Stale(value)
        | WorthQueryContributionComposedOrchestrationOutcome::RebindRequired(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Unsupported(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Failed(value) => {
            (value.kind(), value.linked_artifacts().clone())
        }
    };
    let mut explanation = WorthQueryRecoveryExplanation::new_with_source_family(
        crate::ordinary_outcome::WorthQueryOrdinaryCheckedTopology::contribution_composed(
            map_contribution_kind(kind),
            linked,
            None,
        ),
        WorthQueryRecoverySourceFamily::ContributionComposed,
    )
    .with_evidence_strength(evidence_strength)
    .with_diagnostic_context(diagnostic_context_for_stop_kind(contribution_stop_kind(
        kind,
    )));
    explanation = explanation.with_aspect_posture(if value_has_retained_aspect_record(checked) {
        WorthQueryRecoveryAspectPosture::RetainedContractAndCoverage
    } else {
        WorthQueryRecoveryAspectPosture::CategoryScopedAspectComposition
    });
    if kind == WorthQueryContributionComposedOrchestrationCheckedKind::Stale {
        explanation = explanation
            .with_basis_posture(WorthQueryRecoveryBasisPosture::StaleBasis)
            .with_support_context(support_context_for_stale_basis());
    }
    if let Some(descriptor) = primary_checked_intent_descriptor(checked) {
        explanation = explanation.with_contribution_intent_descriptor(descriptor.clone());
    }
    Ok(explanation)
}

fn contribution_proof_explanation<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    proof: &WorthQueryContributionComposedOrchestrationTranscript<D, I>,
    evidence_strength: WorthQueryRecoveryEvidenceStrength,
) -> WorthQueryRecoveryExplanation {
    let basis_posture = match proof.outcome() {
        WorthQueryContributionComposedOrchestrationOutcome::Stale(_) => {
            WorthQueryRecoveryBasisPosture::StaleBasis
        }
        WorthQueryContributionComposedOrchestrationOutcome::RebindRequired(_) => {
            WorthQueryRecoveryBasisPosture::Unknown
        }
        _ => WorthQueryRecoveryBasisPosture::Unknown,
    };
    let conflict_posture = match proof.composition_classification() {
        Some(WorthQueryContributionComposedClassification::PartiallyAdmitted)
        | Some(WorthQueryContributionComposedClassification::MaterializationFailedAfterAdmission) => {
            WorthQueryRecoveryConflictPosture::MixedContributionFailure
        }
        _ => WorthQueryRecoveryConflictPosture::None,
    };
    let mut explanation =
        base_contribution_explanation(evidence_strength, basis_posture, conflict_posture)
            .with_aspect_posture(if proof.declaration().aspect_record().is_some() {
                WorthQueryRecoveryAspectPosture::RetainedContractAndCoverage
            } else {
                WorthQueryRecoveryAspectPosture::CategoryScopedAspectComposition
            });
    if let Some(descriptor) = primary_intent_descriptor(proof.intent_results()) {
        explanation = explanation.with_contribution_intent_descriptor(descriptor.clone());
    }
    if basis_posture == WorthQueryRecoveryBasisPosture::StaleBasis {
        explanation = explanation.with_support_context(support_context_for_stale_basis());
    }
    explanation
}

fn base_contribution_explanation(
    evidence_strength: WorthQueryRecoveryEvidenceStrength,
    basis_posture: WorthQueryRecoveryBasisPosture,
    conflict_posture: WorthQueryRecoveryConflictPosture,
) -> WorthQueryRecoveryExplanation {
    WorthQueryRecoveryExplanation::new_with_source_family(
        crate::ordinary_outcome::WorthQueryOrdinaryCheckedTopology::contribution_composed(
            crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Failed,
            crate::binding_pipeline::WorthQueryBindingLinkedArtifacts::new(),
            None,
        ),
        WorthQueryRecoverySourceFamily::ContributionComposed,
    )
    .with_evidence_strength(evidence_strength)
    .with_basis_posture(basis_posture)
    .with_conflict_posture(conflict_posture)
}

fn primary_intent_descriptor(
    intent_results: &[crate::contribution_composed_orchestration::WorthQueryContributionComposedIntentResult],
) -> Option<&WorthQueryContributionComposedIntentRequestDescriptor> {
    intent_results
        .iter()
        .find(|value| {
            value.classification() != WorthQueryContributionComposedIntentClassification::Admitted
        })
        .or_else(|| intent_results.first())
        .map(|value| value.request())
}

fn value_has_retained_aspect_record<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    checked: &WorthQueryContributionComposedOrchestrationChecked<D, I>,
) -> bool {
    checked_posture(checked)
        .and_then(|value| value.declaration_aspect_record())
        .is_some()
}

fn primary_checked_intent_descriptor<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    checked: &WorthQueryContributionComposedOrchestrationChecked<D, I>,
) -> Option<&WorthQueryContributionComposedIntentRequestDescriptor> {
    checked_posture(checked).and_then(|value| value.primary_intent_descriptor())
}

fn checked_posture<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    checked: &WorthQueryContributionComposedOrchestrationChecked<D, I>,
) -> Option<
    &crate::contribution_composed_orchestration::WorthQueryContributionComposedOrchestrationPosture<
        D,
        I,
    >,
> {
    match checked {
        WorthQueryContributionComposedOrchestrationOutcome::EvidenceIdentityDerivationDenied(_) => {
            None
        }

        WorthQueryContributionComposedOrchestrationOutcome::Bound(_) => None,
        WorthQueryContributionComposedOrchestrationOutcome::Deferred(value)
        | WorthQueryContributionComposedOrchestrationOutcome::DeclarationDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::ContributionDenied(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Stale(value)
        | WorthQueryContributionComposedOrchestrationOutcome::RebindRequired(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Unsupported(value)
        | WorthQueryContributionComposedOrchestrationOutcome::Failed(value) => Some(value),
    }
}

fn map_contribution_kind(
    kind: WorthQueryContributionComposedOrchestrationCheckedKind,
) -> crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind {
    match kind {
        WorthQueryContributionComposedOrchestrationCheckedKind::Deferred => {
            crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Deferred
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::DeclarationDenied => {
            crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::DeclarationDenied
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::ContributionDenied => {
            crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::ContributionDenied
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Stale => {
            crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Stale
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::RebindRequired => {
            crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::RebindRequired
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Unsupported => {
            crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Unsupported
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Failed => {
            crate::ordinary_outcome::WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Failed
        }
    }
}

fn contribution_stop_kind(
    kind: WorthQueryContributionComposedOrchestrationCheckedKind,
) -> WorthQueryRecoveryStopKind {
    match kind {
        WorthQueryContributionComposedOrchestrationCheckedKind::Deferred => {
            WorthQueryRecoveryStopKind::Deferred
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::DeclarationDenied => {
            WorthQueryRecoveryStopKind::DeclarationDenied
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::ContributionDenied => {
            WorthQueryRecoveryStopKind::ContributionDenied
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Stale => {
            WorthQueryRecoveryStopKind::Stale
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::RebindRequired => {
            WorthQueryRecoveryStopKind::RebindRequired
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Unsupported => {
            WorthQueryRecoveryStopKind::Unsupported
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Failed => {
            WorthQueryRecoveryStopKind::Failed
        }
    }
}
