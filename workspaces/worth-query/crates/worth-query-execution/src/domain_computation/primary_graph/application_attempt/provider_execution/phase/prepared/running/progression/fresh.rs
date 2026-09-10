use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::{
    progression_denied, WorthQueryProviderProgressionOutcome,
};
use crate::domain_computation::primary_graph::application_attempt::{
    provider_recomparison::recover_equivalent_commit_evidence,
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationStaleAttempt,
    WorthQueryCommittedReceiptProjection,
};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryPrimaryGraphProvider, WorthQueryProviderIdempotencyResolution,
};
use crate::domain_computation::{
    WorthQueryDecisionFactRequest, WorthQueryDecisionReadSetFreshnessOutcome,
    WorthQueryFreshDecisionReadSet, WorthQuerySessionBoundReadsAndEffects,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialStage as DenialStage;

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryRegisteredEquivalentCommitReceiptPermit
{
    provider_session:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
}

impl WorthQueryRegisteredEquivalentCommitReceiptPermit {
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

pub(super) enum WorthQueryProviderReadSetProgression<'run> {
    Fresh(WorthQueryFreshProviderAttempt<'run>),
    Terminal(WorthQueryProviderProgressionOutcome),
}

pub(super) struct WorthQueryFreshProviderAttempt<'run> {
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    read_set: WorthQueryFreshDecisionReadSet,
}

impl<'run> WorthQueryFreshProviderAttempt<'run> {
    pub(super) fn progress_invariant(
        self,
        steps: Vec<crate::domain_computation::WorthQueryProvisionalEffectStep>,
        provider: &std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    ) -> Result<
        crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
        WorthQueryProviderProgressionOutcome,
    > {
        super::invariant::progress_invariant_candidate(self.staged, self.read_set, steps, provider)
    }
}

pub(super) fn compare_provider_read_set<'run, Schema, Operation, Input, Scope>(
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    requests: Vec<WorthQueryDecisionFactRequest>,
    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
) -> WorthQueryProviderReadSetProgression<'run>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    if let Some(outcome) = registered_idempotency_outcome(&staged, authority) {
        let _ = staged.abort();
        return WorthQueryProviderReadSetProgression::Terminal(outcome);
    }
    let receipt = match staged.read_authority().capture_decision_read_set(requests) {
        Ok(receipt) => receipt,
        Err(failure) => {
            let _ = staged.abort();
            return WorthQueryProviderReadSetProgression::Terminal(decision_read_set_denied(
                failure,
            ));
        }
    };
    match staged.read_authority().compare_decision_read_set(receipt) {
        Ok(WorthQueryDecisionReadSetFreshnessOutcome::Fresh(read_set)) => {
            WorthQueryProviderReadSetProgression::Fresh(WorthQueryFreshProviderAttempt {
                staged,
                read_set,
            })
        }
        Ok(WorthQueryDecisionReadSetFreshnessOutcome::Stale(stale)) => {
            resolve_stale_provider_read_set(staged, stale.stale_fact_count(), authority)
        }
        Err(failure) => {
            let _ = staged.abort();
            WorthQueryProviderReadSetProgression::Terminal(decision_read_set_denied(failure))
        }
    }
}

fn decision_read_set_denied(
    failure: crate::domain_computation::WorthQueryDecisionReadSetFailure,
) -> WorthQueryProviderProgressionOutcome {
    match failure.kind() {
        crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryProviderProgressionOutcome::Denied(
            WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                DenialStage::DecisionReadSet,
                maximum_active_snapshots,
            ),
        ),
        crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RetentionCapacityExhausted => {
            WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                    DenialStage::DecisionReadSet,
                ),
            )
        }
        _ => progression_denied(DenialStage::DecisionReadSet),
    }
}

fn resolve_stale_provider_read_set<'run, Schema, Operation, Input, Scope>(
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    stale_fact_count: usize,
    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
) -> WorthQueryProviderReadSetProgression<'run>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let outcome = registered_idempotency_outcome(&staged, authority).unwrap_or_else(|| {
        WorthQueryProviderProgressionOutcome::Stale(WorthQueryApplicationStaleAttempt::new(
            stale_fact_count,
        ))
    });
    let _ = staged.abort();
    WorthQueryProviderReadSetProgression::Terminal(outcome)
}

fn registered_idempotency_outcome<Schema, Operation, Input, Scope>(
    staged: &WorthQuerySessionBoundReadsAndEffects<'_>,
    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
) -> Option<WorthQueryProviderProgressionOutcome>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let proof = match authority.authorization.authorize_application_commit(
        authority.application(),
        authority.admission(),
        authority.coordination(),
    ) {
        Ok(proof) => proof,
        Err(denial) => {
            return Some(
                crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::progression_from_authorization_denial(
                    denial,
                    DenialStage::DecisionReadSet,
                ),
            )
        }
    };
    let provider_session = staged.provider_session_terminal_binding();
    let resolution = proof.govern((), |()| {
        authority
            .provider()
            .resolve_application_idempotency(&provider_session)
    });
    match resolution {
        Err(((), denial)) => Some(
            crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::progression_from_authorization_denial(
                denial,
                DenialStage::DecisionReadSet,
            ),
        ),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Equivalent(receipt))) => {
            Some(match WorthQueryCommittedReceiptProjection::resolve(receipt) {
                Ok(projection) => {
                    let receipt = WorthQueryApplicationCommitReceipt::from_registered_equivalent(
                        WorthQueryRegisteredEquivalentCommitReceiptPermit::mint(provider_session),
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
                    WorthQueryProviderProgressionOutcome::AlreadyCommitted(receipt)
                }
                Err(_) => progression_denied(DenialStage::Idempotency),
            })
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Drift)) => {
            Some(WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            ))
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Unpublished)) => {
            Some(progression_denied(DenialStage::Idempotency))
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Absent)) => None,
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        })) => Some(WorthQueryProviderProgressionOutcome::Denied(
            WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                DenialStage::Idempotency,
                maximum_active_snapshots,
            ),
        )),
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::Unavailable)) => {
            Some(progression_denied(DenialStage::Idempotency))
        }
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::RetentionCapacityExhausted)) => {
            Some(WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::RetentionIdentityExhausted)) => {
            Some(WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::SnapshotIdentityExhausted)) => {
            Some(WorthQueryProviderProgressionOutcome::Denied(
                WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
    }
}
