use super::*;
use crate::domain_computation::primary_graph as graph;
use graph::application_attempt::provider_compare_denial::provider_session_execution_denied;
use graph::provider::WorthQueryProviderIdempotencyResolutionDenial as IdempotencyDenial;
use graph::WorthQueryApplicationCommitDenialStage as ExecutionDenialStage;

pub(super) fn resolve_retained_idempotency<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    product: &crate::basis::WorthQueryProductBranchLease,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    aftermath_causality: Option<&WorthQueryPendingAftermathCausality>,
) -> Option<WorthQueryApplicationCommitOutcome>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let product = product.publication_binding();
    let command_lane = application
        .primary_provider
        .correction_commit_lane(&idempotency);
    let _command = command_lane.as_ref().map(|lane| lane.enter());
    let commit_lane = application
        .primary_provider
        .application_branch_commit_lane(product.observation());

    let coordination = commit_lane.enter();
    let proof =
        match application.authorize_retained_idempotency(execution, admission, &coordination) {
            Ok(proof) => proof,
            Err(denial) => {
                return Some(commit_outcome_from_authorization_denial(
                    denial,
                    DenialStage::DecisionReadSet,
                ))
            }
        };
    match proof.govern((), |()| {
        application
            .primary_provider
            .resolve_idempotency_binding_at_product(execution, idempotency, &product)
    }) {
        Err(((), denial)) => Some(commit_outcome_from_authorization_denial(
            denial,
            DenialStage::DecisionReadSet,
        )),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::PreparationBusy))
        | Ok(Err(IdempotencyDenial::PreparationBusy)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_preparation_busy(),
            ))
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Absent)) => None,
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Readmitted(receipt))) => Some(
            if super::super::super::aftermath_resolution::recovered_answers_aftermath(
                aftermath_causality,
                &receipt,
            ) {
                WorthQueryApplicationCommitOutcome::RecoveredCommitted(receipt)
            } else {
                WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
                )
            },
        ),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Drift)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            ))
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::PreparationPending { handle, stage })) => {
            Some(
                WorthQueryApplicationCommitOutcome::ProductPreparationPending(
                    application
                        .primary_provider
                        .preparation_pending(&product, handle, stage),
                ),
            )
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::RecoveredUnpublished { handle })) => Some(
            match application
                .primary_provider
                .recovered_unpublished_application(&product, handle)
            {
                Ok(effects) => WorthQueryApplicationCommitOutcome::ProductUnpublished(effects),
                Err(_) => WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::idempotency_preparation_busy(),
                ),
            },
        ),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Unpublished)) => {
            Some(denied(DenialStage::Idempotency))
        }
        Ok(Err(IdempotencyDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        })) => Some(WorthQueryApplicationCommitOutcome::Denied(
            WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                DenialStage::Idempotency,
                maximum_active_snapshots,
            ),
        )),
        Ok(Err(IdempotencyDenial::ExecutionDenied(kind))) => Some(
            WorthQueryApplicationCommitOutcome::Denied(provider_session_execution_denied(
                kind,
                ExecutionDenialStage::Idempotency,
                "pending publication execution refused",
            )),
        ),
        Ok(Err(IdempotencyDenial::Unavailable)) => Some(denied(DenialStage::Idempotency)),
        Ok(Err(IdempotencyDenial::ForeignAffinity)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_foreign_affinity(),
            ))
        }
        Ok(Err(IdempotencyDenial::RecordedIntentUnverifiable)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_unverifiable(),
            ))
        }
        Ok(Err(IdempotencyDenial::RetentionCapacityExhausted)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
        Ok(Err(IdempotencyDenial::RetentionIdentityExhausted)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
        Ok(Err(IdempotencyDenial::SnapshotIdentityExhausted)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
    }
}
