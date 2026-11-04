use super::commit_resolution::{finish_authorized_compare, WorthQueryAuthorizedCompareContext};
use crate::domain_computation::primary_graph as graph;
use graph::application_attempt::provider_compare_denial::provider_session_execution_denied;
use graph::application_attempt::provider_execution::outcome::{
    progression_denied, progression_from_authorization_denial, WorthQueryProviderProgressionOutcome,
};
use graph::application_attempt::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialStage as DenialStage,
};
use graph::provider::WorthQueryProviderIdempotencyResolution;
use graph::provider::WorthQueryProviderIdempotencyResolutionDenial as IdempotencyDenial;
use graph::WorthQueryApplicationCommitDenialStage as ExecutionDenialStage;

pub(super) struct WorthQueryAuthorizedProviderCommit<
    'run,
    'a,
    'provider,
    Schema,
    Operation,
    Input,
    Scope,
> {
    candidate: crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
    authority: &'a super::WorthQueryApplicationCommitProgressionAuthority<
        'a,
        'provider,
        Schema,
        Operation,
        Input,
        Scope,
    >,
    dispatch_outbox:
        Option<crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord>,
}

fn authorize_provider_commit<'run, 'a, 'provider, Schema, Operation, Input, Scope>(
    candidate: crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
    authority: &'a super::WorthQueryApplicationCommitProgressionAuthority<
        'a,
        'provider,
        Schema,
        Operation,
        Input,
        Scope,
    >,
    dispatch_outbox: Option<
        crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
    >,
) -> WorthQueryAuthorizedProviderCommit<'run, 'a, 'provider, Schema, Operation, Input, Scope> {
    WorthQueryAuthorizedProviderCommit {
        candidate,
        authority,
        dispatch_outbox,
    }
}

fn resolve_authorized_provider_commit<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    authorized: WorthQueryAuthorizedProviderCommit<'_, '_, '_, Schema, Operation, Input, Scope>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> WorthQueryProviderProgressionOutcome
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let WorthQueryAuthorizedProviderCommit {
        candidate,
        authority,
        dispatch_outbox,
    } = authorized;
    let proof = match authority.authorization.authorize_application_commit(
        execution,
        authority.application(),
        authority.admission(),
        authority.coordination(),
    ) {
        Ok(proof) => proof,
        Err(denial) => {
            candidate.discard();
            return progression_from_authorization_denial(denial, DenialStage::DecisionReadSet);
        }
    };
    match proof.govern(candidate, |candidate| {
        resolve_idempotency_under_authority(
            execution,
            candidate,
            authority,
            dispatch_outbox,
            allocation_policy,
        )
    }) {
        Ok(outcome) => outcome,
        Err((candidate, denial)) => {
            candidate.discard();
            progression_from_authorization_denial(denial, DenialStage::DecisionReadSet)
        }
    }
}

pub(super) fn authorize_and_resolve_provider_commit<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    candidate: crate::domain_computation::WorthQueryInvariantApprovedProposedState<'_>,
    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
    dispatch_outbox: Option<
        crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
    >,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> WorthQueryProviderProgressionOutcome
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    resolve_authorized_provider_commit(
        execution,
        authorize_provider_commit(candidate, authority, dispatch_outbox),
        allocation_policy,
    )
}

fn resolve_idempotency_under_authority<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    candidate: crate::domain_computation::WorthQueryInvariantApprovedProposedState<'_>,
    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
    dispatch_outbox: Option<
        crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
    >,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> WorthQueryProviderProgressionOutcome
where
    Input: Clone + Send + Sync + 'static,
{
    let provider_session = candidate.provider_session_terminal_binding();
    match authority
        .provider()
        .resolve_application_idempotency(execution, &provider_session)
    {
        Ok(WorthQueryProviderIdempotencyResolution::PreparationBusy)
        | Err(IdempotencyDenial::PreparationBusy) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_preparation_busy(),
            )
        }
        Ok(WorthQueryProviderIdempotencyResolution::Absent) => {
            let control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(
                authority.admission().publication_request(), allocation_policy,
            );
            let context = WorthQueryAuthorizedCompareContext::from_progression(
                authority,
                dispatch_outbox,
                provider_session,
            );
            let compared = candidate.compare_and_commit_with(control.policy(), |read_authority, fresh| {
                authority.provider().recompare_application_read_set(execution,
                    read_authority, fresh,
                    crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(
                        control.policy(), Some(authority.admission().publication_request()),
                    ),
                )
            });
            finish_authorized_compare(execution, compared, context)
        }
        Ok(WorthQueryProviderIdempotencyResolution::Readmitted(receipt)) => {
            candidate.discard();
            if graph::application_attempt::provider_execution::aftermath_resolution::recovered_answers_aftermath(authority.aftermath_causality(), &receipt) {
                WorthQueryProviderProgressionOutcome::RecoveredCommitted(receipt)
            } else {
                WorthQueryProviderProgressionOutcome::Denied(WorthQueryApplicationCommitDenial::idempotency_intent_drift())
            }
        }
        Ok(WorthQueryProviderIdempotencyResolution::Drift) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            )
        }
        Ok(WorthQueryProviderIdempotencyResolution::PreparationPending { handle, stage }) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::ProductPreparationPending(
                authority
                    .application()
                    .primary_provider
                    .preparation_pending(&authority.product.publication_binding(), handle, stage),
            )
        }
        Ok(WorthQueryProviderIdempotencyResolution::RecoveredUnpublished { handle }) => {
            candidate.discard();
            match authority
                .application()
                .primary_provider
                .recovered_unpublished_application(&authority.product.publication_binding(), handle)
            {
                Ok(effects) => WorthQueryProviderProgressionOutcome::ProductUnpublished(effects),
                Err(_) => WorthQueryProviderProgressionOutcome::Denied(
                    WorthQueryApplicationCommitDenial::idempotency_preparation_busy(),
                ),
            }
        }
        Ok(WorthQueryProviderIdempotencyResolution::Unpublished) => {
            candidate.discard();
            progression_denied(DenialStage::Idempotency)
        }
        Err(IdempotencyDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        }) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                    DenialStage::Idempotency,
                    maximum_active_snapshots,
                ),
            )
        }
        Err(IdempotencyDenial::ExecutionDenied(kind)) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(provider_session_execution_denied(
                kind,
                ExecutionDenialStage::Idempotency,
                "pending publication execution refused",
            ))
        }
        Err(IdempotencyDenial::Unavailable) => {
            candidate.discard();
            progression_denied(DenialStage::Idempotency)
        }
        Err(IdempotencyDenial::ForeignAffinity) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_foreign_affinity(),
            )
        }
        Err(IdempotencyDenial::RecordedIntentUnverifiable) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_unverifiable(),
            )
        }
        Err(IdempotencyDenial::RetentionCapacityExhausted) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                    DenialStage::Idempotency,
                ),
            )
        }
        Err(IdempotencyDenial::RetentionIdentityExhausted) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            )
        }
        Err(IdempotencyDenial::SnapshotIdentityExhausted) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            )
        }
    }
}
