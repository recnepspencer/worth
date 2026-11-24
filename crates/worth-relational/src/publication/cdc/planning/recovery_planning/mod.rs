use worth_relational_publication::internal_port::publication::cdc::data::subscriber_failure::SubscriberStreamFailureOwnerPort as _;
mod checkpoint_projection;
mod envelope_sources;
mod request_validation;

use crate::publication::cdc::data::{
    SubscriberRecoveryDecision, SubscriberRecoveryPlan, SubscriberRecoverySource,
    SubscriberResumeRequest, SubscriberStreamFailure,
};
use crate::publication::cdc::diagnostics::{
    continuation_assessment_artifact, recovery_decision_artifact,
};
use crate::publication::cdc::planning::checkpoint_resolution::{
    resolve_checkpoint, resolve_latest_available_checkpoint,
};
use crate::publication::cdc::planning::{
    assess_subscriber_continuity, disposition_for_assessment, select_execution_envelopes,
};
use crate::runtime::RelationalRuntime;

use self::checkpoint_projection::{
    latest_available_assessment, latest_available_checkpoint_for_recovery,
};
use self::envelope_sources::load_available_envelopes;
use self::request_validation::validate_resume_request;

pub(crate) fn plan_subscriber_recovery(
    runtime: &RelationalRuntime,
    request: SubscriberResumeRequest,
) -> Result<
    (
        SubscriberRecoveryPlan,
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    ),
    SubscriberStreamFailure,
> {
    validate_resume_request(runtime, &request)?;

    let available_envelope_source = load_available_envelopes(runtime, request.checkpoint())?;
    let (start_after_position, mut diagnostics) = resolve_checkpoint(
        runtime,
        request.checkpoint(),
        available_envelope_source.durable_envelopes(),
    )?;
    request_validation::validate_checkpoint_contract(runtime, &request)?;

    let use_durable_source = available_envelope_source.is_durable();
    let available_envelopes = available_envelope_source.into_envelopes();
    let selected_envelopes = select_execution_envelopes(
        &available_envelopes,
        start_after_position,
        request.max_commits(),
    );
    let prior_proof = request
        .checkpoint()
        .map(|checkpoint| checkpoint.normalized_continuation_proof().clone())
        .unwrap_or_default();
    let resume_basis_descriptor_semantics_version = request
        .checkpoint()
        .map(|checkpoint| checkpoint.descriptor_semantics_version())
        .or_else(|| {
            selected_envelopes
                .last()
                .map(|envelope| envelope.descriptor_semantics_version)
        })
        .unwrap_or_default();
    let continuation_assessment = assess_subscriber_continuity(
        runtime,
        &selected_envelopes,
        request.subscriber_contract(),
        &prior_proof,
        resume_basis_descriptor_semantics_version,
    )
    .map_err(|failure| {
        SubscriberStreamFailure::new(
            failure.class,
            failure.detail,
            resolve_latest_available_checkpoint(runtime),
            failure.diagnostics,
        )
    })?;
    let latest_available_assessment = latest_available_assessment(
        runtime,
        &available_envelopes,
        &selected_envelopes,
        request.subscriber_contract(),
        &prior_proof,
        resume_basis_descriptor_semantics_version,
    )?;
    let decision = SubscriberRecoveryDecision {
        disposition: disposition_for_assessment(
            start_after_position,
            continuation_assessment.continuation_outcome(),
        ),
        source: if use_durable_source {
            SubscriberRecoverySource::DurableCanonicalRecovery
        } else {
            SubscriberRecoverySource::InMemoryHistory
        },
        start_after_position,
    };
    let latest_available_checkpoint = latest_available_checkpoint_for_recovery(
        request.subscriber_contract().contract_id.clone(),
        &available_envelopes,
        latest_available_assessment
            .as_ref()
            .unwrap_or(&continuation_assessment),
    )
    .or_else(|| request.checkpoint().cloned());
    diagnostics.push(continuation_assessment_artifact(
        &request,
        &continuation_assessment,
    ));
    diagnostics.push(recovery_decision_artifact(&decision));
    Ok((
        SubscriberRecoveryPlan::new(
            request,
            decision,
            latest_available_checkpoint,
            start_after_position,
            selected_envelopes,
            continuation_assessment,
        ),
        diagnostics,
    ))
}
