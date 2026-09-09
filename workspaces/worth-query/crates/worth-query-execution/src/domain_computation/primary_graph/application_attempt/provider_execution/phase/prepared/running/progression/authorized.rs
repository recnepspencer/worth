use crate::domain_computation::primary_graph::application_attempt::{
    provider_recomparison::recover_equivalent_commit_evidence,
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialStage as DenialStage, WorthQueryApplicationCommitReceipt,
    WorthQueryCommittedReceiptProjection,
};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::aftermath_resolution::resolve_exact_committed_aftermath;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::{
    progression_denied, progression_from_authorization_denial,
    WorthQueryProviderProgressionOutcome,
};
use super::commit_resolution::{finish_authorized_compare, WorthQueryAuthorizedCompareContext};
use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolution;

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryManagedEquivalentCommitReceiptPermit
{
    provider_session:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
}

impl WorthQueryManagedEquivalentCommitReceiptPermit {
    fn mint(
        provider_session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
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
        authority.serialization(),
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
        Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::ActiveSnapshotCapacityExhausted {
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
        Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::Unavailable) => {
            candidate.discard();
            progression_denied(DenialStage::Idempotency)
        }
        Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::RetentionCapacityExhausted) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                    DenialStage::Idempotency,
                ),
            )
        }
        Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::RetentionIdentityExhausted) => {
            candidate.discard();
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            )
        }
        Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::SnapshotIdentityExhausted) => {
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
    receipt: crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication,
    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
    provider_session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
) -> WorthQueryProviderProgressionOutcome
where
    Input: Clone + Send + Sync + 'static,
{
    match resolve_exact_committed_aftermath(
        authority.provider(),
        authority.aftermath_causality(),
        &receipt,
    ) {
        Ok(causality) => match WorthQueryCommittedReceiptProjection::resolve(receipt) {
            Ok(projection) => {
                let receipt = WorthQueryApplicationCommitReceipt::from_managed_equivalent(
                    WorthQueryManagedEquivalentCommitReceiptPermit::mint(provider_session),
                    projection,
                    recover_equivalent_commit_evidence(
                        authority.admission().mutation_preconditions(),
                    ),
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
        },
        Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        }) => WorthQueryProviderProgressionOutcome::Denied(
            WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                DenialStage::Idempotency,
                maximum_active_snapshots,
            ),
        ),
        Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::Unavailable) => {
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            )
        }
        Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionCapacityExhausted) => {
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                    DenialStage::Idempotency,
                ),
            )
        }
        Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionIdentityExhausted) => {
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            )
        }
        Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::SnapshotIdentityExhausted) => {
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            )
        }
    }
}
