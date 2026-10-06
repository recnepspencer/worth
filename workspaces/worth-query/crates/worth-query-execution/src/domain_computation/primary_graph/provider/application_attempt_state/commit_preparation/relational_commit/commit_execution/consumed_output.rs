use worth_relational::facade::snapshots::SnapshotHandle;

use crate::domain_computation::primary_graph::invariant_projection::{
    ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
};
use crate::domain_computation::primary_graph::provider::{
    session_commit::provider_failure, WorthQueryPrimaryGraphApplicationAttempt,
    WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::{
    WorthQueryProviderSessionCommitStop, WorthQueryProviderSessionProtocolStage,
};

pub(super) struct VerifiedConsumedOutputPublication {
    pub(super) publication_admission: crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    pub(super) required_prerequisites:
        Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
}

enum PrecommitOutputStop {
    SelectedUnavailable,
    Verification(ConsumedOutputVerificationStop),
    Changed,
}

/// Recheck every shared upstream observation at the exact selected precommit
/// root, on the request's meter, which registration opened and paid the
/// consumed edges' backing from, through all later preparation.
pub(super) fn verify(
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &mut WorthQueryPrimaryGraphApplicationAttempt,
    before: &SnapshotHandle,
) -> Result<VerifiedConsumedOutputPublication, WorthQueryProviderSessionCommitStop> {
    let owner = &provider.graph.source_owner.invalidation_owner;
    let context = attempt.take_required_output_demand();
    let mut admission = attempt.take_request_admission();
    if !attempt.consumed_outputs().is_empty() {
        // A restored output's row is recorded before the check reads it, on
        // the same meter as the check.
        attempt
            .consumed_outputs()
            .iter()
            .try_for_each(|consumed| {
                consumed
                    .establish_restored_before_commit(&provider.graph.source_owner, &mut admission)
                    .map_err(PrecommitOutputStop::Verification)
            })
            .and_then(|()| provider
        .graph
        .with_runtime(|runtime| {
            let selected = runtime
                .read_truth()
                .positioned_snapshot(before)
                .map_err(|_| PrecommitOutputStop::SelectedUnavailable)?;
            let current = ConsumedOutputEvidence::verify_many_with_admission(
                attempt.consumed_outputs(),
                owner,
                runtime,
                before,
                &selected,
                &mut admission,
            )
            .map_err(PrecommitOutputStop::Verification)?;
            if current != ConsumedOutputVerification::Current {
                return Err(PrecommitOutputStop::Changed);
            }
            Ok::<(), PrecommitOutputStop>(())
        }))
        .map_err(|stop| match stop {
            PrecommitOutputStop::Verification(
                ConsumedOutputVerificationStop::RetryCurrentness(cause),
            ) => WorthQueryProviderSessionCommitStop::Deferred(
                crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
                    crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::SourceCurrentnessRaced(cause),
                    "consumed output currentness changed during verification",
                ),
            ),
            PrecommitOutputStop::Verification(ConsumedOutputVerificationStop::Interrupted(
                event,
            )) => WorthQueryProviderSessionCommitStop::ControlStopped(
                super::stops::interruption_control_stopped(event),
            ),
            other => {
                let detail = match other {
                    PrecommitOutputStop::SelectedUnavailable => {
                        "selected native root is unavailable for output evidence"
                    }
                    PrecommitOutputStop::Verification(
                        ConsumedOutputVerificationStop::WorkExhausted,
                    ) => "consumed output currentness exceeded publication work",
                    PrecommitOutputStop::Verification(
                        ConsumedOutputVerificationStop::PendingUpstream,
                    ) => "consumed output has pending upstream source",
                    PrecommitOutputStop::Verification(
                        ConsumedOutputVerificationStop::Unavailable,
                    ) => "consumed output source verification is unavailable",
                    PrecommitOutputStop::Verification(
                        ConsumedOutputVerificationStop::Interrupted(_),
                    ) => unreachable!("an interruption is a control stop above"),
                    PrecommitOutputStop::Changed => {
                        "consumed output changed before application publication"
                    }
                    PrecommitOutputStop::Verification(
                        ConsumedOutputVerificationStop::RetryCurrentness(_),
                    ) => unreachable!("currentness race is deferred above"),
                };
                WorthQueryProviderSessionCommitStop::PreEffectDenied(provider_failure(
                    WorthQueryProviderSessionProtocolStage::Commit,
                    detail,
                ))
            }
        })?;
    }
    let required_prerequisites = if let Some(context) = context {
        let prepared = context
            .prepare_prerequisites(
                attempt.consumed_outputs().iter().map(|evidence| evidence.identity()),
                &mut admission,
            )
            .map_err(|denial| {
                WorthQueryProviderSessionCommitStop::Deferred(
                    crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
                        crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::RequiredPrerequisitePending(denial.kind()),
                        "required output prerequisites could not be admitted before publication",
                    ),
                )
            })?;
        Some(prepared)
    } else {
        None
    };
    Ok(VerifiedConsumedOutputPublication {
        publication_admission: admission,
        required_prerequisites,
    })
}
