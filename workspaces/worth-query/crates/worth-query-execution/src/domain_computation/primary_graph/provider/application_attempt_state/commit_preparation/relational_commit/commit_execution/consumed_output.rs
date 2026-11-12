use worth_relational::facade::snapshots::SnapshotHandle;

use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerification;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop;
use crate::domain_computation::primary_graph::provider::session_commit::provider_failure;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::WorthQueryProviderSessionCommitStop;
use crate::domain_computation::WorthQueryProviderSessionProtocolStage;

pub(super) struct VerifiedConsumedOutputPublication {
    pub(super) required_prerequisites:
        Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
}

#[derive(Debug)]
pub(super) enum ConsumedOutputPreparationDenial {
    Prerequisite(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial),
    SelectedUnavailable,
    Verification(ConsumedOutputVerificationStop),
    Changed,
}

/// Recheck every shared upstream observation at the exact selected precommit
/// root, on the request's meter, which registration opened and paid the
/// consumed edges' backing from, through all later preparation.
pub(super) fn verify(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &mut WorthQueryPrimaryGraphApplicationAttempt,
    before: &SnapshotHandle,
) -> Result<VerifiedConsumedOutputPublication, ConsumedOutputPreparationDenial> {
    let owner = &provider.graph.source_owner.invalidation_owner;
    let context = attempt.take_required_output_demand();

    if !attempt.consumed_outputs().is_empty() {
        // A restored output's row is recorded before the check reads it, on
        // the same meter as the check.
        attempt
            .consumed_outputs()
            .iter()
            .try_for_each(|consumed| {
                consumed
                    .establish_restored_before_commit(execution, &provider.graph.source_owner)
                    .map_err(ConsumedOutputPreparationDenial::Verification)
            })
            .and_then(|()| {
                provider.graph.with_runtime(execution, |runtime| {
                    let selected = runtime
                        .read_truth()
                        .positioned_snapshot(before)
                        .map_err(|_| ConsumedOutputPreparationDenial::SelectedUnavailable)?;
                    let current = ConsumedOutputEvidence::verify_many_with_admission(
                        execution,
                        attempt.consumed_outputs(),
                        owner,
                        runtime,
                        before,
                        &selected,
                    )
                    .map_err(ConsumedOutputPreparationDenial::Verification)?;
                    if current != ConsumedOutputVerification::Current {
                        return Err(ConsumedOutputPreparationDenial::Changed);
                    }
                    Ok::<(), ConsumedOutputPreparationDenial>(())
                })
            })?;
    }
    let required_prerequisites = if let Some(context) = context {
        let prepared = context
            .prepare_prerequisites(attempt.consumed_outputs().iter(), owner)
            .map_err(ConsumedOutputPreparationDenial::Prerequisite)?;
        Some(prepared)
    } else {
        None
    };
    Ok(VerifiedConsumedOutputPublication {
        required_prerequisites,
    })
}

pub(super) fn verification_stop(
    stop: ConsumedOutputVerificationStop,
) -> WorthQueryProviderSessionCommitStop {
    memory_stop(ConsumedOutputPreparationDenial::Verification(stop))
}

pub(super) fn memory_stop(
    stop: ConsumedOutputPreparationDenial,
) -> WorthQueryProviderSessionCommitStop {
    match stop {
            ConsumedOutputPreparationDenial::Prerequisite(denial) => WorthQueryProviderSessionCommitStop::Deferred(
                crate::domain_computation::WorthQueryProviderSessionCommitDeferred::required_prerequisite(
                    denial,
                    "required output prerequisites could not be admitted before publication",
                ),
            ),
            ConsumedOutputPreparationDenial::Verification(ConsumedOutputVerificationStop::ManagedIndex(cause)) => WorthQueryProviderSessionCommitStop::PreEffectDenied(
                provider_failure(WorthQueryProviderSessionProtocolStage::Commit, "managed Source consistency refused")
                    .with_managed_index_denial(cause)),
            ConsumedOutputPreparationDenial::Verification(
                ConsumedOutputVerificationStop::RetryCurrentness(cause),
            ) => WorthQueryProviderSessionCommitStop::Deferred(
                crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
                    crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::SourceCurrentnessRaced(cause),
                    "consumed output currentness changed during verification",
                ),
            ),
            ConsumedOutputPreparationDenial::Verification(ConsumedOutputVerificationStop::Interrupted(
                event,
            )) => WorthQueryProviderSessionCommitStop::ControlStopped(
                super::stops::interruption_control_stopped(event),
            ),
            ConsumedOutputPreparationDenial::Verification(ConsumedOutputVerificationStop::Execution(cause)) => crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop(cause, None),
            ConsumedOutputPreparationDenial::Verification(ConsumedOutputVerificationStop::SourceQueryAuthorization(kind)) => {
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Authorization;
                let detail = "source query authorization is no longer current";
                match kind {
                    Authorization::Stopped(Err(control)) => WorthQueryProviderSessionCommitStop::ControlStopped(
                        crate::domain_computation::WorthQueryProviderSessionCommitControlStopped::new(control, detail),
                    ),
                    Authorization::Stopped(Ok(denial)) => WorthQueryProviderSessionCommitStop::PreEffectDenied(
                        crate::domain_computation::WorthQueryProviderSessionFailure::checked_read_stopped(
                            Ok(denial), WorthQueryProviderSessionProtocolStage::Commit, detail,
                        ).with_source_query_authorization_stop(kind),
                    ),
                    Authorization::Refused(_) => WorthQueryProviderSessionCommitStop::PreEffectDenied(
                        provider_failure(WorthQueryProviderSessionProtocolStage::Commit, detail)
                            .with_source_query_authorization_stop(kind),
                    ),
                }
            }
            ConsumedOutputPreparationDenial::Verification(ConsumedOutputVerificationStop::CapacityExhausted(stop)) => capacity_denied(stop),
            other => {
                let detail = match other {
                    ConsumedOutputPreparationDenial::Prerequisite(_) => unreachable!("prerequisite cause routed above"),
                    ConsumedOutputPreparationDenial::Verification(ConsumedOutputVerificationStop::ManagedIndex(_)) => unreachable!("managed source cause retained above"),
                    ConsumedOutputPreparationDenial::Verification(ConsumedOutputVerificationStop::Execution(_)) => unreachable!("request execution routed above"),
                    ConsumedOutputPreparationDenial::Verification(ConsumedOutputVerificationStop::SourceQueryAuthorization(_)) => unreachable!("source query authorization routed above"),
                    ConsumedOutputPreparationDenial::SelectedUnavailable => {
                        "selected native root is unavailable for output evidence"
                    }
                    ConsumedOutputPreparationDenial::Verification(
                        ConsumedOutputVerificationStop::WorkExhausted,
                    ) => "consumed output currentness exceeded publication work",
                    ConsumedOutputPreparationDenial::Verification(ConsumedOutputVerificationStop::CapacityExhausted(_)) => unreachable!("capacity evidence retained above"),
                    ConsumedOutputPreparationDenial::Verification(
                        ConsumedOutputVerificationStop::PendingUpstream,
                    ) => "consumed output has pending upstream source",
                    ConsumedOutputPreparationDenial::Verification(
                        ConsumedOutputVerificationStop::Unavailable,
                    ) => "consumed output source verification is unavailable",
                    ConsumedOutputPreparationDenial::Verification(
                        ConsumedOutputVerificationStop::Interrupted(_),
                    ) => unreachable!("an interruption is a control stop above"),
                    ConsumedOutputPreparationDenial::Changed => {
                        "consumed output changed before application publication"
                    }
                    ConsumedOutputPreparationDenial::Verification(
                        ConsumedOutputVerificationStop::RetryCurrentness(_),
                    ) => unreachable!("currentness race is deferred above"),
                };
                WorthQueryProviderSessionCommitStop::PreEffectDenied(provider_failure(
                    WorthQueryProviderSessionProtocolStage::Commit,
                    detail,
                ))
            }
    }
}

fn capacity_denied(
    stop: crate::domain_computation::primary_graph::VerificationCapacityStop,
) -> WorthQueryProviderSessionCommitStop {
    WorthQueryProviderSessionCommitStop::PreEffectDenied(
        provider_failure(
            WorthQueryProviderSessionProtocolStage::Commit,
            "consumed output currentness exceeded publication capacity",
        )
        .with_output_verification_capacity_stop(stop),
    )
}

#[cfg(test)]
mod capacity_tests;
