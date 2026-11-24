use crate::publication::cdc::data::{
    SubscriberCheckpoint, SubscriberStreamBatch, SubscriberStreamFailure,
};
use crate::publication::cdc::planning::checkpoint_basis_from_patch_position;
use crate::runtime::RelationalRuntime;
#[cfg(test)]
use crate::schema::data::SchemaBoundaryFingerprint;
#[cfg(test)]
use std::collections::BTreeSet;

pub(crate) fn execute_subscriber_stream(
    runtime: &RelationalRuntime,
    plan: crate::publication::cdc::data::SubscriberRecoveryPlan,
    diagnostics: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
) -> Result<SubscriberStreamBatch, SubscriberStreamFailure> {
    let continuation_assessment = &plan.continuation_assessment;
    let patches = plan
        .selected_envelopes
        .iter()
        .map(|commit| commit.published_patch())
        .collect::<Vec<_>>();
    let next_checkpoint = patches
        .last()
        .and_then(|patch| checkpoint_basis_from_patch_position(runtime, patch.position));
    let next_checkpoint = next_checkpoint.map(|basis| {
        let descriptor_semantics_version = continuation_assessment
            .normalized_continuation_proof()
            .descriptor_semantics_version();
        SubscriberCheckpoint::from_basis_with_assessment(
            basis,
            plan.request.subscriber_contract().contract_id.clone(),
            continuation_assessment,
            descriptor_semantics_version,
        )
    });
    let latest_commit_id = runtime.history().latest_canonical_commit_id();

    Ok(SubscriberStreamBatch {
        resumed_from: plan.request.checkpoint().cloned(),
        next_checkpoint,
        latest_available_checkpoint: plan.latest_available_checkpoint,
        recovery_decision: plan.decision,
        latest_commit_id,
        continuation: continuation_assessment.clone(),
        patches,
        diagnostics,
    })
}

#[cfg(test)]
pub(crate) fn collect_crossed_boundaries(
    selected_envelopes: &[crate::history::data::PositionedCanonicalCommit],
) -> Vec<SchemaBoundaryFingerprint> {
    let mut boundaries = Vec::new();
    let mut seen = BTreeSet::new();
    for envelope in selected_envelopes {
        if let Some(descriptor) = &envelope.schema_continuation_descriptor {
            let fingerprint = descriptor.boundary_fingerprint;
            if seen.insert(fingerprint) {
                boundaries.push(fingerprint);
            }
        }
    }
    boundaries
}
