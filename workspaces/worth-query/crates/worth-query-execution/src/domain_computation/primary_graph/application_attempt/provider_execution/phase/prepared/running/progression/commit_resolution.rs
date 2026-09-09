//! Resolving one authorized compare-and-commit into a progression outcome.

use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolution;
use crate::domain_computation::WorthQueryProviderCompareAndCommitOutcome;

use crate::domain_computation::primary_graph::application_attempt::provider_recomparison::certify_provider_recomparison;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitDenialStage as DenialStage, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationCommitDeferred, WorthQueryApplicationSettlementDeferred,
    WorthQueryApplicationStaleAttempt,
    WorthQueryCommittedReceiptProjection,
    WorthQueryPendingApplicationCommitReceipt, WorthQueryApplicationCommitRecoveryKind,
    WorthQueryApplicationUnresolvedCommitEvidence,
};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::aftermath_resolution::resolve_exact_committed_aftermath;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::{progression_denied, WorthQueryProviderProgressionOutcome};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::recovery_evidence;

/// Everything needed to turn a provider compare-and-commit answer into a
/// progression outcome, including the dispatch anchor the commit carried.
pub(super) struct WorthQueryAuthorizedCompareContext<'a> {
    provider: &'a crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    branch: &'a worth_relational::facade::history::BranchId,
    preconditions:
        &'a crate::domain_computation::primary_graph::application_attempt::precondition_binding::WorthQueryBoundMutationPreconditions,
    canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
    dispatch_outbox: Option<WorthQueryDispatchOutboxRecord>,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
    authority_binding: super::super::super::WorthQueryApplicationCommitAuthorityBinding,
    provider_session:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
}

/// One-shot permission for the receipt owner to join a fresh provider commit
/// to the exact current terminal session.
pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryFreshCommitReceiptPermit
{
    provider_session:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
}

impl WorthQueryFreshCommitReceiptPermit {
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

impl<'a> WorthQueryAuthorizedCompareContext<'a> {
    pub(super) fn from_progression<Schema, Operation, Input, Scope>(
        authority: &'a super::WorthQueryApplicationCommitProgressionAuthority<
            'a,
            '_,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        dispatch_outbox: Option<WorthQueryDispatchOutboxRecord>,
        provider_session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    ) -> Self
    where
        Input: Clone + Send + Sync + 'static,
    {
        let admission = authority.admission();
        Self {
            provider: authority.provider(),
            idempotency: authority.idempotency(),
            branch: admission.graph_work().branch().relational(),
            preconditions: admission.mutation_preconditions(),
            canonical_work: admission.canonical_work(),
            dispatch_outbox,
            aftermath_causality: authority.aftermath_causality().cloned(),
            authority_binding:
                super::super::super::WorthQueryApplicationCommitAuthorityBinding::from_admission(
                    admission,
                    authority.idempotency(),
                ),
            provider_session,
        }
    }
}

pub(super) fn finish_authorized_compare(
    compared: WorthQueryProviderCompareAndCommitOutcome,
    context: WorthQueryAuthorizedCompareContext<'_>,
) -> WorthQueryProviderProgressionOutcome {
    match compared {
        WorthQueryProviderCompareAndCommitOutcome::Committed(committed_session) => {
            if !context
                .provider_session
                .same_session(committed_session.terminal_binding())
            {
                return WorthQueryProviderProgressionOutcome::Indeterminate(
                    recovery_evidence::unknown_commit_recovery_evidence(
                        "provider commit disposition belongs to another session",
                    ),
                );
            }
            let resolved = context
                .provider
                .observe_completed_application_for_session(committed_session.terminal_binding())
                .ok_or(WorthQueryCommittedComponentResolutionDenial::Unavailable(
                    "typed provider terminal evidence could not be resolved after commit",
                ))
                .and_then(|receipt| resolve_committed_components(&context, receipt));
            resolved.map_or_else(
                |denial| {
                    WorthQueryProviderProgressionOutcome::Indeterminate(
                        committed_component_recovery_evidence(denial),
                    )
                },
                |resolved| seal_committed_outcome(context, resolved),
            )
        }
        WorthQueryProviderCompareAndCommitOutcome::Stale(stale) => {
            WorthQueryProviderProgressionOutcome::Stale(WorthQueryApplicationStaleAttempt::new(
                stale.stale_fact_count(),
            ))
        }
        WorthQueryProviderCompareAndCommitOutcome::ProductUnpublished(unpublished) => {
            WorthQueryProviderProgressionOutcome::ProductUnpublished(unpublished)
        }
        WorthQueryProviderCompareAndCommitOutcome::ProductStale(stale) => {
            WorthQueryProviderProgressionOutcome::ProductStale(stale)
        }
        WorthQueryProviderCompareAndCommitOutcome::Denied(denial) => {
            provider_compare_denied(denial)
        }
        WorthQueryProviderCompareAndCommitOutcome::Deferred(deferred) => {
            WorthQueryProviderProgressionOutcome::Deferred(
                WorthQueryApplicationCommitDeferred::from_provider_session(deferred),
            )
        }
        WorthQueryProviderCompareAndCommitOutcome::ControlStopped(stopped) => {
            match stopped.kind() {
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled => {
                    WorthQueryProviderProgressionOutcome::Cancelled
                }
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut => {
                    WorthQueryProviderProgressionOutcome::TimedOut
                }
            }
        }
        WorthQueryProviderCompareAndCommitOutcome::SettlementDeferred(deferred) => {
            WorthQueryProviderProgressionOutcome::SettlementDeferred(
                WorthQueryApplicationSettlementDeferred::from_provider_session(
                    deferred,
                    context.idempotency,
                    context.branch.clone(),
                ),
            )
        }
        WorthQueryProviderCompareAndCommitOutcome::Indeterminate(failure) => {
            resolve_indeterminate_commit(context, failure)
        }
    }
}

fn provider_compare_denied(
    denial: crate::domain_computation::WorthQueryProviderCompareAndCommitDenial,
) -> WorthQueryProviderProgressionOutcome {
    let (capacity, retention, identity, prepared_root_budget) = match denial {
        crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
            failure,
        ) => match failure.kind() {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => (Some(maximum_active_snapshots), false, None, None),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted => (None, true, None, None),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted => (None, false, Some(WorthQueryCommitIdentityExhaustion::Retention), None),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted => (None, false, Some(WorthQueryCommitIdentityExhaustion::Snapshot), None),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::CandidateIdentityExhausted => (None, false, Some(WorthQueryCommitIdentityExhaustion::Candidate), None),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::PreparedRootBudgetExhausted { maximum_bytes, required_bytes } => (None, false, None, Some((maximum_bytes, required_bytes))),
            _ => (None, false, None, None),
        },
        crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::DecisionReadSet(
            failure,
        ) => match failure.kind() {
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => (Some(maximum_active_snapshots), false, None, None),
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RetentionCapacityExhausted => {
                (None, true, None, None)
            }
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RetentionIdentityExhausted => {
                (None, false, Some(WorthQueryCommitIdentityExhaustion::Retention), None)
            }
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::SnapshotIdentityExhausted => {
                (None, false, Some(WorthQueryCommitIdentityExhaustion::Snapshot), None)
            }
            _ => (None, false, None, None),
        },
    };
    match (capacity, retention, identity, prepared_root_budget) {
        (Some(maximum_active_snapshots), _, _, _) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                DenialStage::ProviderCommit,
                maximum_active_snapshots,
            ),
        ),
        (None, true, _, _) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                DenialStage::ProviderCommit,
            ),
        ),
        (None, false, Some(WorthQueryCommitIdentityExhaustion::Retention), _) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                DenialStage::ProviderCommit,
            ),
        ),
        (None, false, Some(WorthQueryCommitIdentityExhaustion::Snapshot), _) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                DenialStage::ProviderCommit,
            ),
        ),
        (None, false, Some(WorthQueryCommitIdentityExhaustion::Candidate), _) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::candidate_identity_exhausted(
                DenialStage::ProviderCommit,
            ),
        ),
        (None, false, None, Some((maximum_bytes, required_bytes))) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::prepared_root_budget_exhausted(
                DenialStage::ProviderCommit,
                maximum_bytes,
                required_bytes,
            ),
        ),
        (None, false, None, None) => progression_denied(DenialStage::ProviderCommit),
    }
}

#[derive(Clone, Copy)]
enum WorthQueryCommitIdentityExhaustion {
    Retention,
    Snapshot,
    Candidate,
}

/// A commit whose answer never arrived is re-read through idempotency.
///
/// The recovery kind is carried, not re-derived: a session that failed on the
/// commit path demands commit recovery even when the record is now visible.
fn resolve_indeterminate_commit(
    context: WorthQueryAuthorizedCompareContext<'_>,
    failure: crate::domain_computation::provider_session::WorthQueryProviderSessionFailure,
) -> WorthQueryProviderProgressionOutcome {
    let evidence = WorthQueryApplicationUnresolvedCommitEvidence::from_provider_session_failure(
        WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired,
        &failure,
    );
    match context
        .provider
        .resolve_idempotency_binding(context.idempotency, context.branch)
    {
        Ok(WorthQueryProviderIdempotencyResolution::Equivalent(receipt)) => {
            match resolve_committed_components(&context, receipt) {
                Ok(resolved) => seal_committed_outcome(context, resolved),
                Err(denial) => WorthQueryProviderProgressionOutcome::Indeterminate(
                    committed_component_recovery_evidence(denial),
                ),
            }
        }
        Ok(WorthQueryProviderIdempotencyResolution::Absent) => {
            WorthQueryProviderProgressionOutcome::Aborted
        }
        Ok(WorthQueryProviderIdempotencyResolution::Drift) | Err(_) => {
            WorthQueryProviderProgressionOutcome::Indeterminate(evidence)
        }
    }
}

struct WorthQueryResolvedCommitComponents {
    projection: WorthQueryCommittedReceiptProjection,
    causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality,
    >,
}

fn committed_component_recovery_evidence(
    denial: WorthQueryCommittedComponentResolutionDenial,
) -> WorthQueryApplicationUnresolvedCommitEvidence {
    match denial {
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
        ) => recovery_evidence::typed_commit_recovery_evidence(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            "committed aftermath recovery requires snapshot-capacity readmission",
        ),
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::Unavailable,
        ) => recovery_evidence::unknown_commit_recovery_evidence(
            "committed aftermath causality could not be recovered",
        ),
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionCapacityExhausted,
        ) => recovery_evidence::typed_commit_recovery_evidence(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted,
            "committed aftermath recovery requires retention-capacity readmission",
        ),
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionIdentityExhausted,
        ) => recovery_evidence::typed_commit_recovery_evidence(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted,
            "committed aftermath recovery exhausted retention identity space",
        ),
        WorthQueryCommittedComponentResolutionDenial::Aftermath(
            crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::SnapshotIdentityExhausted,
        ) => recovery_evidence::typed_commit_recovery_evidence(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted,
            "committed aftermath recovery exhausted snapshot identity space",
        ),
        WorthQueryCommittedComponentResolutionDenial::Unavailable(detail) => {
            recovery_evidence::unknown_commit_recovery_evidence(detail)
        }
    }
}

enum WorthQueryCommittedComponentResolutionDenial {
    Aftermath(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial),
    Unavailable(&'static str),
}

fn resolve_committed_components(
    context: &WorthQueryAuthorizedCompareContext<'_>,
    receipt: crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication,
) -> Result<WorthQueryResolvedCommitComponents, WorthQueryCommittedComponentResolutionDenial> {
    let causality = resolve_exact_committed_aftermath(
        context.provider,
        context.aftermath_causality.as_ref(),
        &receipt,
    )
    .map_err(WorthQueryCommittedComponentResolutionDenial::Aftermath)?;
    let projection = WorthQueryCommittedReceiptProjection::resolve(receipt).map_err(|_| {
        WorthQueryCommittedComponentResolutionDenial::Unavailable(
            "committed dispatch outbox binding was denied",
        )
    })?;
    if projection
        .committed_dispatch_outbox()
        .map(|binding| binding.record())
        != context.dispatch_outbox.as_ref()
    {
        return Err(WorthQueryCommittedComponentResolutionDenial::Unavailable(
            "committed dispatch outbox evidence does not match the admitted attempt",
        ));
    }
    Ok(WorthQueryResolvedCommitComponents {
        projection,
        causality,
    })
}

fn seal_committed_outcome(
    context: WorthQueryAuthorizedCompareContext<'_>,
    resolved: WorthQueryResolvedCommitComponents,
) -> WorthQueryProviderProgressionOutcome {
    let permit = WorthQueryFreshCommitReceiptPermit::mint(context.provider_session);
    let Some(receipt) = WorthQueryPendingApplicationCommitReceipt::from_projection(
        permit,
        resolved.projection,
        certify_provider_recomparison(context.preconditions),
        context.canonical_work,
        context.authority_binding,
    ) else {
        return WorthQueryProviderProgressionOutcome::Indeterminate(
            recovery_evidence::unknown_commit_recovery_evidence(
                "committed provider evidence belongs to another session",
            ),
        );
    };
    WorthQueryProviderProgressionOutcome::Committed(
        receipt.with_aftermath_causality(resolved.causality),
    )
}
