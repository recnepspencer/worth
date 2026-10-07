use super::commit_resolution::{finish_authorized_compare, WorthQueryAuthorizedCompareContext};
use crate::domain_computation::primary_graph as graph;
use graph::application_attempt::provider_compare_denial::provider_session_kind_denied;
use graph::application_attempt::provider_execution::aftermath_resolution::resolve_exact_committed_aftermath;
use graph::application_attempt::provider_execution::outcome::{
    progression_denied, progression_from_authorization_denial, WorthQueryProviderProgressionOutcome,
};
use graph::application_attempt::{
    provider_recomparison::recover_equivalent_commit_evidence,
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialStage as DenialStage, WorthQueryApplicationCommitReceipt,
    WorthQueryCommittedReceiptProjection,
};
use graph::provider::WorthQueryProviderIdempotencyResolution;
use graph::provider::WorthQueryProviderIdempotencyResolutionDenial as IdempotencyDenial;
use graph::WorthQueryApplicationCommitDenialStage as ExecutionDenialStage;

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryManagedEquivalentCommitReceiptPermit
{
    provider_session:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
}

impl WorthQueryManagedEquivalentCommitReceiptPermit {
    fn mint(
        provider_session: crate::domain_computation::provider_session::
            WorthQueryProviderSessionTerminalBinding,
    ) -> Self {
        Self { provider_session }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn into_provider_session(
        self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding {
        self.provider_session
    }
}

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
    authorized: WorthQueryAuthorizedProviderCommit<'_, '_, '_, Schema, Operation, Input, Scope>,
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
        resolve_idempotency_under_authority(candidate, authority, dispatch_outbox)
    }) {
        Ok(outcome) => outcome,
        Err((candidate, denial)) => {
            candidate.discard();
            progression_from_authorization_denial(denial, DenialStage::DecisionReadSet)
        }
    }
}

pub(super) fn authorize_and_resolve_provider_commit<Schema, Operation, Input, Scope>(
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
) -> WorthQueryProviderProgressionOutcome
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    resolve_authorized_provider_commit(authorize_provider_commit(
        candidate,
        authority,
        dispatch_outbox,
    ))
}

fn resolve_idempotency_under_authority<Schema, Operation, Input, Scope>(
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
) -> WorthQueryProviderProgressionOutcome
where
    Input: Clone + Send + Sync + 'static,
{
    let provider_session = candidate.provider_session_terminal_binding();
    match authority
        .provider()
        .resolve_application_idempotency(&provider_session)
    {
        Ok(WorthQueryProviderIdempotencyResolution::Absent) => finish_authorized_compare(
            candidate.compare_and_commit(),
            WorthQueryAuthorizedCompareContext::from_progression(
                authority,
                dispatch_outbox,
                provider_session,
            ),
        ),
        Ok(WorthQueryProviderIdempotencyResolution::Equivalent(receipt)) => {
            candidate.discard();
            resolve_equivalent_commit(receipt, authority, provider_session)
        }
        Ok(WorthQueryProviderIdempotencyResolution::Drift) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            )
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
            WorthQueryProviderProgressionOutcome::Denied(provider_session_kind_denied(
                kind,
                ExecutionDenialStage::Idempotency,
                "pending publication execution refused",
            ))
        }
        Err(IdempotencyDenial::Unavailable) => {
            candidate.discard();
            progression_denied(DenialStage::Idempotency)
        }
        Err(IdempotencyDenial::WindowExpired) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_window_expired(),
            )
        }
        Err(IdempotencyDenial::CommittedReceiptNotRetained { commit }) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_receipt_not_retained(commit),
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

fn resolve_equivalent_commit<Schema, Operation, Input, Scope>(
    receipt: graph::provider::WorthQueryPrimaryGraphCommittedApplication,
    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
    provider_session: crate::domain_computation::provider_session::
        WorthQueryProviderSessionTerminalBinding,
) -> WorthQueryProviderProgressionOutcome
where
    Input: Clone + Send + Sync + 'static,
{
    let Ok(causality) =
        resolve_exact_committed_aftermath(authority.aftermath_causality(), &receipt)
    else {
        return WorthQueryProviderProgressionOutcome::Denied(
            WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
        );
    };
    match WorthQueryCommittedReceiptProjection::resolve(receipt) {
        Ok(projection) => {
            let receipt = WorthQueryApplicationCommitReceipt::from_managed_equivalent(
                WorthQueryManagedEquivalentCommitReceiptPermit::mint(provider_session),
                projection,
                recover_equivalent_commit_evidence(authority.admission().mutation_preconditions()),
                authority.admission().canonical_work(),
                WorthQueryApplicationCommitAuthorityBinding::from_admission(
                    authority.admission(),
                    authority.idempotency(),
                ),
            );
            WorthQueryProviderProgressionOutcome::AlreadyCommitted(
                receipt.with_aftermath_causality(causality),
            )
        }
        Err(_) => progression_denied(DenialStage::Idempotency),
    }
}
