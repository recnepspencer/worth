use crate::application::{WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput};
use crate::binding_pipeline::{
    WorthQueryBindingLinkedArtifacts, WorthQueryBindingNarrowingDecision,
    WorthQueryBindingRequestDescriptor, WorthQueryBindingWitnessCheck,
};
use crate::domain_installation::WorthQueryInstalledDomainExecutionDrift;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::support::transcript_digest;
use crate::continuation_pipeline::transcript::{
    WorthQueryContinuationExecutionTranscript, WorthQueryPreparedContinuationTranscript,
};
use crate::continuation_pipeline::{
    WorthQueryContinuationExecutionOutcome, WorthQueryPreparedContinuationOutcome,
};

pub(super) fn prepared_installed_authority_drift_transcript<D, I>(
    request: WorthQueryBindingRequestDescriptor,
    linked: WorthQueryBindingLinkedArtifacts,
    drift: WorthQueryInstalledDomainExecutionDrift,
) -> Result<WorthQueryPreparedContinuationTranscript<D, I>, IdentityDerivationDenial>
where
    I: WorthQueryDeclarationInput<D>,
    D: crate::application::WorthQueryDomainEntryMarker,
{
    let reason = drift.to_string();
    Ok(WorthQueryPreparedContinuationTranscript::new(
        request,
        WorthQueryPreparedContinuationOutcome::InstalledAuthorityDrift(drift),
        vec![WorthQueryBindingWitnessCheck::failed(
            "installed_authority_freshness",
            &reason,
        )],
        vec![WorthQueryBindingNarrowingDecision::new(
            "continuation preparation stopped before binding or planning because installed authority was not current",
        )],
        transcript_digest(
            "prepared_continuation",
            I::Family::semantic_family_key(),
            &linked,
            "installed_authority_drift",
        )?,
        linked,
    ))
}

pub(super) fn execution_installed_authority_drift_transcript<D, I>(
    request: WorthQueryBindingRequestDescriptor,
    linked: WorthQueryBindingLinkedArtifacts,
    drift: WorthQueryInstalledDomainExecutionDrift,
) -> Result<WorthQueryContinuationExecutionTranscript<D, I>, IdentityDerivationDenial>
where
    I: WorthQueryDeclarationInput<D>,
    D: crate::application::WorthQueryDomainEntryMarker,
{
    let reason = drift.to_string();
    Ok(WorthQueryContinuationExecutionTranscript::new(
        request,
        WorthQueryContinuationExecutionOutcome::InstalledAuthorityDrift(drift),
        vec![WorthQueryBindingWitnessCheck::failed(
            "installed_authority_freshness",
            &reason,
        )],
        transcript_digest(
            "execute_prepared_continuation",
            I::Family::semantic_family_key(),
            &linked,
            "installed_authority_drift",
        )?,
        linked,
    ))
}
