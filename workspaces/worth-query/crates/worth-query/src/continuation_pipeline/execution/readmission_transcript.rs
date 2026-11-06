use crate::application::{
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
};
use crate::binding_pipeline::{WorthQueryBindingRequestDescriptor, WorthQueryBindingWitnessCheck};
use crate::continuation_pipeline::{
    WorthQueryContinuationExecutionOutcome, WorthQueryContinuationExecutionReadmissionStop,
    WorthQueryContinuationExecutionReadmissionStopKind, WorthQueryContinuationExecutionTranscript,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::support::transcript_digest;

pub(super) fn transcript_from_readmission_denial<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    request_descriptor: WorthQueryBindingRequestDescriptor,
    linked: crate::binding_pipeline::WorthQueryBindingLinkedArtifacts,
    stop: WorthQueryContinuationExecutionReadmissionStop,
) -> Result<WorthQueryContinuationExecutionTranscript<D, I>, IdentityDerivationDenial> {
    Ok(match stop.kind() {
        WorthQueryContinuationExecutionReadmissionStopKind::StaleBasis => transcript(
            request_descriptor,
            linked,
            WorthQueryContinuationExecutionOutcome::Stale(stop),
            "basis_freshness",
            "retained continuation basis evidence is stale",
            "stale",
        )?,
        WorthQueryContinuationExecutionReadmissionStopKind::AsyncRequestDrift => transcript(
            request_descriptor,
            linked,
            WorthQueryContinuationExecutionOutcome::AsyncRequestDrift(stop),
            "async_request_alignment",
            "current continuation async request identity drifted from the retained request",
            "async_request_drift",
        )?,
        WorthQueryContinuationExecutionReadmissionStopKind::ReplayDrift => transcript(
            request_descriptor,
            linked,
            WorthQueryContinuationExecutionOutcome::ReplayDrift(stop),
            "replay_alignment",
            "current continuation replay identity drifted from the retained replay witness",
            "replay_drift",
        )?,
        WorthQueryContinuationExecutionReadmissionStopKind::PolicyRemaskDrift => transcript(
            request_descriptor,
            linked,
            WorthQueryContinuationExecutionOutcome::RemaskDrift(stop),
            "remask_alignment",
            "current continuation meaning was remasked before execution",
            "remask_drift",
        )?,
        WorthQueryContinuationExecutionReadmissionStopKind::PreviewCrossedResidue => transcript(
            request_descriptor,
            linked,
            WorthQueryContinuationExecutionOutcome::PreviewCrossedResidue(stop),
            "preview_residue_alignment",
            "current continuation crossed preview residue before execution",
            "preview_crossed_residue",
        )?,
        WorthQueryContinuationExecutionReadmissionStopKind::StaleCompletion => transcript(
            request_descriptor,
            linked,
            WorthQueryContinuationExecutionOutcome::StaleCompletion(stop),
            "completion_freshness",
            "current continuation completion posture is stale at execution time",
            "stale_completion",
        )?,
        WorthQueryContinuationExecutionReadmissionStopKind::BasisMismatch => transcript(
            request_descriptor,
            linked,
            WorthQueryContinuationExecutionOutcome::BasisMismatch(stop),
            "basis_alignment",
            "current lower-runtime basis evidence drifted from the retained continuation basis",
            "basis_mismatch",
        )?,
        WorthQueryContinuationExecutionReadmissionStopKind::LowerBindingMismatch => transcript(
            request_descriptor,
            linked,
            WorthQueryContinuationExecutionOutcome::LowerBindingMismatch(stop),
            "lower_binding_alignment",
            "current lower-runtime binding drifted from the retained continuation binding",
            "lower_binding_mismatch",
        )?,
        WorthQueryContinuationExecutionReadmissionStopKind::AuthorityMismatch => transcript(
            request_descriptor,
            linked,
            WorthQueryContinuationExecutionOutcome::AuthorityMismatch(stop),
            "authority_alignment",
            "current lower-runtime authority no longer matches retained continuation authority",
            "authority_mismatch",
        )?,
    })
}

fn transcript<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    request_descriptor: WorthQueryBindingRequestDescriptor,
    linked: crate::binding_pipeline::WorthQueryBindingLinkedArtifacts,
    outcome: WorthQueryContinuationExecutionOutcome<D, I>,
    failed_check: &'static str,
    failure_reason: &'static str,
    outcome_token: &'static str,
) -> Result<WorthQueryContinuationExecutionTranscript<D, I>, IdentityDerivationDenial> {
    Ok(WorthQueryContinuationExecutionTranscript::new(
        request_descriptor,
        outcome,
        vec![
            WorthQueryBindingWitnessCheck::passed("world_alignment"),
            WorthQueryBindingWitnessCheck::passed("execution_support"),
            WorthQueryBindingWitnessCheck::failed(failed_check, failure_reason),
        ],
        transcript_digest(
            "execute_prepared_continuation",
            I::Family::semantic_family_key(),
            &linked,
            outcome_token,
        )?,
        linked,
    ))
}
