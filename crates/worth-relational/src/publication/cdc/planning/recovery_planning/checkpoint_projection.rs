use crate::history::data::PositionedCanonicalCommit;
use crate::publication::cdc::data::{
    NormalizedContinuationProof, SubscriberCheckpoint, SubscriberContinuationAssessment,
    SubscriberContractDeclaration, SubscriberStreamFailure,
};
use crate::publication::cdc::planning::assess_subscriber_continuity;
use crate::publication::cdc::planning::checkpoint_resolution::checkpoint_basis_from_envelope;
use crate::runtime::RelationalRuntime;
use crate::schema::data::DescriptorSemanticsVersion;
use worth_relational_publication::internal_port::publication::cdc::data::subscriber_checkpoint::SubscriberCheckpointOwnerPort as _;

pub(super) fn latest_available_assessment(
    runtime: &RelationalRuntime,
    available_envelopes: &[PositionedCanonicalCommit],
    selected_envelopes: &[PositionedCanonicalCommit],
    subscriber_contract: &SubscriberContractDeclaration,
    prior_proof: &NormalizedContinuationProof,
    resume_basis_descriptor_semantics_version: DescriptorSemanticsVersion,
) -> Result<Option<SubscriberContinuationAssessment>, SubscriberStreamFailure> {
    if selected_envelopes.len() == available_envelopes.len() {
        return Ok(None);
    }

    assess_subscriber_continuity(
        runtime,
        available_envelopes,
        subscriber_contract,
        prior_proof,
        resume_basis_descriptor_semantics_version,
    )
    .map(Some)
}

pub(super) fn latest_available_checkpoint_for_recovery(
    subscriber_contract_id: String,
    available_envelopes: &[PositionedCanonicalCommit],
    assessment: &SubscriberContinuationAssessment,
) -> Option<SubscriberCheckpoint> {
    let basis = checkpoint_basis_from_envelope(available_envelopes.last()?);
    let descriptor_semantics_version = assessment
        .normalized_continuation_proof()
        .descriptor_semantics_version();
    Some(SubscriberCheckpoint::from_basis_with_assessment(
        basis,
        subscriber_contract_id,
        assessment,
        descriptor_semantics_version,
    ))
}
