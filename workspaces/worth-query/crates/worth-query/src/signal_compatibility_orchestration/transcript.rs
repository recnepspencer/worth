use crate::application::{
    WorthQueryDeclarationBridgeContinuationRequest, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationSignalCompatibilityChecked,
    WorthQueryDeclarationSignalCompatibilityDenialCause, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryInstalledDomainDeclarationContext,
};
use crate::binding_pipeline::{
    WorthQueryBindingLinkedArtifacts, WorthQueryBindingNarrowingDecision,
    WorthQueryBindingRequestDescriptor, WorthQueryBindingWitnessCheck,
};
use crate::continuation_pipeline::{
    prepare_continuation_from_signal_checked_on_handle, WorthQueryPreparedContinuationOutcome,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::lower::{
    checked_and_linked_from_subject, handle_alignment_outcome, linked_from_orchestration_subject,
    orchestration_digest,
};
use super::{
    WorthQuerySignalCompatibilityOrchestration, WorthQuerySignalCompatibilityOrchestrationChecked,
    WorthQuerySignalCompatibilityOrchestrationInput,
    WorthQuerySignalCompatibilityOrchestrationOutcome,
};

pub struct WorthQuerySignalCompatibilityOrchestrationTranscript<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    request: WorthQueryBindingRequestDescriptor,
    outcome: WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>,
    witness_checks: Vec<WorthQueryBindingWitnessCheck>,
    narrowing_decisions: Vec<WorthQueryBindingNarrowingDecision>,
    orchestration_digest: String,
    linked_artifacts: WorthQueryBindingLinkedArtifacts,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        request: WorthQueryBindingRequestDescriptor,
        outcome: WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>,
        witness_checks: Vec<WorthQueryBindingWitnessCheck>,
        narrowing_decisions: Vec<WorthQueryBindingNarrowingDecision>,
        orchestration_digest: String,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
    ) -> Self {
        Self {
            request,
            outcome,
            witness_checks,
            narrowing_decisions,
            orchestration_digest,
            linked_artifacts,
        }
    }

    pub fn request(&self) -> &WorthQueryBindingRequestDescriptor {
        &self.request
    }

    pub fn outcome(&self) -> &WorthQuerySignalCompatibilityOrchestrationOutcome<D, I> {
        &self.outcome
    }

    pub fn witness_checks(&self) -> &[WorthQueryBindingWitnessCheck] {
        &self.witness_checks
    }

    pub fn narrowing_decisions(&self) -> &[WorthQueryBindingNarrowingDecision] {
        &self.narrowing_decisions
    }

    pub fn orchestration_digest(&self) -> &str {
        &self.orchestration_digest
    }

    pub fn linked_artifacts(&self) -> &WorthQueryBindingLinkedArtifacts {
        &self.linked_artifacts
    }

    pub fn into_checked(self) -> WorthQuerySignalCompatibilityOrchestrationChecked<D, I> {
        WorthQuerySignalCompatibilityOrchestrationChecked::new(
            self.outcome,
            self.orchestration_digest,
            self.linked_artifacts,
        )
    }
}

pub(crate) fn orchestrate_signal_compatibility_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    input: WorthQuerySignalCompatibilityOrchestrationInput<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial> {
    let (subject, required_contract, bridge_request) = input.into_parts();
    let request = WorthQueryBindingRequestDescriptor::new(
        I::Family::semantic_family_key(),
        "signal_compatibility_orchestration",
        required_contract.clone(),
    );
    if let Some(outcome) = handle_alignment_outcome(handle, &subject) {
        let linked = linked_from_orchestration_subject(&subject);
        let digest = orchestration_digest("misaligned", &linked, "misaligned");
        return Ok(WorthQuerySignalCompatibilityOrchestrationTranscript::new(
            request,
            outcome,
            vec![WorthQueryBindingWitnessCheck::failed(
                "handle_alignment",
                "the retained signal subject no longer matches this admitted handle",
            )],
            Vec::new(),
            digest,
            linked,
        ));
    }

    let (checked, linked) = checked_and_linked_from_subject(handle, subject);
    let (outcome, witness_checks, narrowing, outcome_token) =
        outcome_from_signal_checked(handle, required_contract, bridge_request, checked)?;
    let digest = orchestration_digest("signal_orchestration", &linked, &outcome_token);
    Ok(WorthQuerySignalCompatibilityOrchestrationTranscript::new(
        request,
        outcome,
        witness_checks,
        narrowing,
        digest,
        linked,
    ))
}

#[cfg(test)]
pub(crate) fn orchestrated_outcome_from_signal_checked_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    required_contract: crate::application::WorthQueryDeclarationAspectContract,
    bridge_request: Option<WorthQueryDeclarationBridgeContinuationRequest>,
    checked: WorthQueryDeclarationSignalCompatibilityChecked<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial> {
    Ok(outcome_from_signal_checked(handle, required_contract, bridge_request, checked)?.0)
}

fn outcome_from_signal_checked<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    required_contract: crate::application::WorthQueryDeclarationAspectContract,
    bridge_request: Option<WorthQueryDeclarationBridgeContinuationRequest>,
    checked: WorthQueryDeclarationSignalCompatibilityChecked<D, I>,
) -> Result<
    (
        WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>,
        Vec<WorthQueryBindingWitnessCheck>,
        Vec<WorthQueryBindingNarrowingDecision>,
        String,
    ),
    IdentityDerivationDenial,
> {
    Ok(match checked {
        WorthQueryDeclarationSignalCompatibilityChecked::Compatible(compatibility) => {
            let mut witness_checks = vec![WorthQueryBindingWitnessCheck::passed(
                "signal_compatibility",
            )];
            if let Some(bridge_request) = bridge_request {
                let proof = prepare_continuation_from_signal_checked_on_handle(
                    handle,
                    WorthQueryDeclarationSignalCompatibilityChecked::Compatible(compatibility),
                    bridge_request,
                    required_contract,
                )?;
                witness_checks.extend(proof.witness_checks().iter().cloned());
                let narrowing = proof.narrowing_decisions().to_vec();
                let checked = proof.into_checked();
                let digest = checked.prepared_digest().to_string();
                return Ok((
                    map_continuation_outcome(checked.into_outcome()),
                    witness_checks,
                    narrowing,
                    digest,
                ));
            }
            let digest = compatibility.signal_compatibility_digest().to_string();
            (
                WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(
                    WorthQuerySignalCompatibilityOrchestration::Compatible(compatibility),
                ),
                witness_checks,
                vec![WorthQueryBindingNarrowingDecision::new(
                    "signal compatibility orchestration stopped at retained compatibility because no continuation request was supplied",
                )],
                digest,
            )
        }
        WorthQueryDeclarationSignalCompatibilityChecked::Deferred(deferred) => (
            WorthQuerySignalCompatibilityOrchestrationOutcome::Deferred(
                deferred.reason().to_string(),
            ),
            vec![WorthQueryBindingWitnessCheck::failed(
                "signal_compatibility",
                deferred.reason(),
            )],
            Vec::new(),
            "deferred".to_string(),
        ),
        WorthQueryDeclarationSignalCompatibilityChecked::Denied(denied) => (
            map_signal_denial(denied.cause(), denied.reason()),
            vec![WorthQueryBindingWitnessCheck::failed(
                "signal_compatibility",
                denied.reason(),
            )],
            Vec::new(),
            format!("denied:{:?}", denied.cause()),
        ),
        WorthQueryDeclarationSignalCompatibilityChecked::Failed(failed) => (
            WorthQuerySignalCompatibilityOrchestrationOutcome::Failed(failed.reason().to_string()),
            vec![WorthQueryBindingWitnessCheck::failed(
                "signal_compatibility",
                failed.reason(),
            )],
            Vec::new(),
            "failed".to_string(),
        ),
    })
}

fn map_signal_denial<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    cause: WorthQueryDeclarationSignalCompatibilityDenialCause,
    reason: &str,
) -> WorthQuerySignalCompatibilityOrchestrationOutcome<D, I> {
    match cause {
        WorthQueryDeclarationSignalCompatibilityDenialCause::SignalBasisMismatch => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::BasisMismatch(reason.to_string())
        }
        WorthQueryDeclarationSignalCompatibilityDenialCause::SignalFamilyUnsupported
        | WorthQueryDeclarationSignalCompatibilityDenialCause::SignalExecutionFamilyUnavailable => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Unsupported(reason.to_string())
        }
        WorthQueryDeclarationSignalCompatibilityDenialCause::MissingRequiredAspect => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::MissingRequiredAspect(
                reason.to_string(),
            )
        }
        WorthQueryDeclarationSignalCompatibilityDenialCause::AspectConflict => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::AspectConflict(reason.to_string())
        }
        WorthQueryDeclarationSignalCompatibilityDenialCause::AuthorityAspectGap => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::AuthorityMismatch(
                reason.to_string(),
            )
        }
        WorthQueryDeclarationSignalCompatibilityDenialCause::EnvelopeNotCoveredForSignalCompatibility
        | WorthQueryDeclarationSignalCompatibilityDenialCause::SignalCompatibilityMismatch => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Denied(reason.to_string())
        }
    }
}

fn map_continuation_outcome<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    outcome: WorthQueryPreparedContinuationOutcome<D, I>,
) -> WorthQuerySignalCompatibilityOrchestrationOutcome<D, I> {
    match outcome {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(
                WorthQuerySignalCompatibilityOrchestration::Prepared(prepared),
            )
        }
        WorthQueryPreparedContinuationOutcome::Ambiguous(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Ambiguous(reason)
        }
        WorthQueryPreparedContinuationOutcome::Unavailable(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Unavailable(reason)
        }
        WorthQueryPreparedContinuationOutcome::WrongWorld(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::WrongWorld(reason)
        }
        WorthQueryPreparedContinuationOutcome::WrongHandle(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::WrongHandle(reason)
        }
        WorthQueryPreparedContinuationOutcome::InstalledAuthorityDrift(drift) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::InstalledAuthorityDrift(drift)
        }
        WorthQueryPreparedContinuationOutcome::Stale(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Stale(reason)
        }
        WorthQueryPreparedContinuationOutcome::RebindRequired(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::RebindRequired(reason)
        }
        WorthQueryPreparedContinuationOutcome::AuthorityMismatch(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::AuthorityMismatch(reason)
        }
        WorthQueryPreparedContinuationOutcome::BasisMismatch(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::BasisMismatch(reason)
        }
        WorthQueryPreparedContinuationOutcome::Unsupported(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Unsupported(reason)
        }
        WorthQueryPreparedContinuationOutcome::Deferred(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Deferred(reason)
        }
        WorthQueryPreparedContinuationOutcome::Denied(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Denied(reason)
        }
        WorthQueryPreparedContinuationOutcome::Failed(reason) => {
            WorthQuerySignalCompatibilityOrchestrationOutcome::Failed(reason)
        }
    }
}
