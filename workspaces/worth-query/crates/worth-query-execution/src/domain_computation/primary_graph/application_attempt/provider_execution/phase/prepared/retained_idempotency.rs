use super::*;
use crate::domain_computation::primary_graph as graph;
use graph::application_attempt::provider_compare_denial::provider_session_kind_denied;
use graph::provider::WorthQueryProviderIdempotencyResolutionDenial as IdempotencyDenial;
use graph::WorthQueryApplicationCommitDenialStage as ExecutionDenialStage;

pub(super) fn resolve_retained_idempotency<Schema, Operation, Input, Scope>(
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
    let commit_lane = application
        .primary_provider
        .application_branch_commit_lane(product.observation());
    let commit_lane = match commit_lane {
        Ok(lane) => lane,
        Err(_) => {
            return Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                    DenialStage::DecisionReadSet,
                ),
            ));
        }
    };
    let coordination = commit_lane.enter();
    let proof = match application.authorize_retained_idempotency(admission, &coordination) {
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
            .resolve_idempotency_binding_at_product(idempotency, &product)
    }) {
        Err(((), denial)) => Some(commit_outcome_from_authorization_denial(
            denial,
            DenialStage::DecisionReadSet,
        )),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Absent)) => None,
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Equivalent(receipt))) => {
            let Ok(causality) = resolve_exact_committed_aftermath(aftermath_causality, &receipt)
            else {
                return Some(WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
                ));
            };
            let projection = match WorthQueryCommittedReceiptProjection::resolve(receipt) {
                Ok(projection) => projection,
                Err(_) => return Some(denied(DenialStage::Idempotency)),
            };
            let receipt = WorthQueryApplicationCommitReceipt::from_early_equivalent(
                WorthQueryEarlyEquivalentCommitReceiptPermit::mint(),
                projection,
                recover_equivalent_commit_evidence(admission.mutation_preconditions()),
                admission.canonical_work(),
                WorthQueryApplicationCommitAuthorityBinding::from_admission(admission, idempotency),
            );
            Some(WorthQueryApplicationCommitOutcome::AlreadyCommitted(
                receipt.with_aftermath_causality(causality),
            ))
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Drift)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            ))
        }
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
            WorthQueryApplicationCommitOutcome::Denied(provider_session_kind_denied(
                kind,
                ExecutionDenialStage::Idempotency,
                "pending publication execution refused",
            )),
        ),
        Ok(Err(IdempotencyDenial::Unavailable)) => Some(denied(DenialStage::Idempotency)),
        Ok(Err(IdempotencyDenial::WindowExpired)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_window_expired(),
            ))
        }
        Ok(Err(IdempotencyDenial::CommittedReceiptNotRetained { commit })) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_receipt_not_retained(commit),
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
