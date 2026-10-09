use super::{DenialStage, WorthQueryProviderProgressionOutcome};

pub(super) fn provider_compare_denied(
    denial: crate::domain_computation::WorthQueryProviderCompareAndCommitDenial,
) -> WorthQueryProviderProgressionOutcome {
    let native_failure = match &denial {
        crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
            failure,
        ) if failure.native_preparation_error().is_some() => Some(failure.clone()),
        _ => None,
    };
    match (classified_provider_denial(denial), native_failure) {
        (WorthQueryProviderProgressionOutcome::Denied(denial), Some(failure)) => {
            WorthQueryProviderProgressionOutcome::Denied(
                denial.with_provider_session_failure(failure),
            )
        }
        (outcome, _) => outcome,
    }
}

fn classified_provider_denial(
    denial: crate::domain_computation::WorthQueryProviderCompareAndCommitDenial,
) -> WorthQueryProviderProgressionOutcome {
    let denial = match denial {
        crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
            failure,
        ) if failure.allocation_denial().is_some() => {
            use worth_execution::ExecutionAllocationDenialKind as Kind;
            return match failure.allocation_denial().expect("matched physical cause").kind() {
                Kind::Cancelled => WorthQueryProviderProgressionOutcome::Cancelled,
                Kind::DeadlineElapsed => WorthQueryProviderProgressionOutcome::TimedOut,
                _ => WorthQueryProviderProgressionOutcome::Denied(
                    crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::provider_session_denied(failure)),
            };
        }
        other => other,
    };
    let provider_detail = match &denial {
        crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
            failure,
        ) => failure.detail().to_owned(),
        crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::DecisionReadSet(
            failure,
        ) => failure.detail().to_owned(),
    };
    if let crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
        failure,
    ) = &denial
    {
        use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial as Denial;
        use crate::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
        match failure.kind() {
            Kind::IndexMaintenanceBudgetExceeded => {
                return WorthQueryProviderProgressionOutcome::Denied(
                    Denial::index_maintenance_budget_exceeded(
                        DenialStage::ProviderCommit,
                        provider_detail,
                    ),
                );
            }
            Kind::IndexGenerationIdentityExhausted => {
                return WorthQueryProviderProgressionOutcome::Denied(
                    Denial::index_generation_identity_exhausted(DenialStage::ProviderCommit),
                );
            }
            _ => {}
        }
    }
    let (capacity, retention, identity) = match denial {
        crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
            failure,
        ) => match failure.kind() {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => (Some(maximum_active_snapshots), false, None),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted => (None, true, None),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted => (None, false, Some(WorthQueryCommitIdentityExhaustion::Retention)),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted => (None, false, Some(WorthQueryCommitIdentityExhaustion::Snapshot)),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::CandidateIdentityExhausted => (None, false, Some(WorthQueryCommitIdentityExhaustion::Candidate)),
            _ => (None, false, None),
        },
        crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::DecisionReadSet(
            failure,
        ) => match failure.kind() {
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => (Some(maximum_active_snapshots), false, None),
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RetentionCapacityExhausted => {
                (None, true, None)
            }
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RetentionIdentityExhausted => {
                (None, false, Some(WorthQueryCommitIdentityExhaustion::Retention))
            }
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::SnapshotIdentityExhausted => {
                (None, false, Some(WorthQueryCommitIdentityExhaustion::Snapshot))
            }
            _ => (None, false, None),
        },
    };
    match (capacity, retention, identity) {
        (Some(maximum_active_snapshots), _, _) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                DenialStage::ProviderCommit,
                maximum_active_snapshots,
            ),
        ),
        (None, true, _) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                DenialStage::ProviderCommit,
            ),
        ),
        (None, false, Some(WorthQueryCommitIdentityExhaustion::Retention)) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                DenialStage::ProviderCommit,
            ),
        ),
        (None, false, Some(WorthQueryCommitIdentityExhaustion::Snapshot)) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                DenialStage::ProviderCommit,
            ),
        ),
        (None, false, Some(WorthQueryCommitIdentityExhaustion::Candidate)) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::candidate_identity_exhausted(
                DenialStage::ProviderCommit,
            ),
        ),
        (None, false, None) => WorthQueryProviderProgressionOutcome::Denied(
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::provider_rejected_with_detail(
                DenialStage::ProviderCommit,
                provider_detail,
            ),
        ),
    }
}

#[derive(Clone, Copy)]
enum WorthQueryCommitIdentityExhaustion {
    Retention,
    Snapshot,
    Candidate,
}

#[cfg(test)]
mod index_preparation_tests {
    use super::*;

    #[test]
    fn pre_effect_index_exhaustion_remains_typed_at_application_boundary() {
        let failure = crate::domain_computation::WorthQueryProviderSessionFailure::new(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::IndexMaintenanceBudgetExceeded,
            crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
            "candidate index work exceeded its finite budget",
            crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
        );
        let outcome = provider_compare_denied(
            crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
                failure,
            ),
        );
        let WorthQueryProviderProgressionOutcome::Denied(denial) = outcome else {
            panic!("pre-effect index refusal must remain a denial");
        };
        assert_eq!(
            denial.kind(),
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialKind::IndexMaintenanceBudgetExceeded,
        );
        assert_eq!(denial.stage(), DenialStage::ProviderCommit);
        assert_eq!(
            denial.detail(),
            Some("candidate index work exceeded its finite budget")
        );
    }
}

#[cfg(test)]
mod native_preparation_tests {
    use super::*;

    #[test]
    fn native_cause_context_and_commit_log_survive_application_kind_mapping() {
        use worth_relational::facade::{
            errors::{ErrorContext, ErrorOperation, RelationalSubsystem},
            mvcc::TransactionCommitError,
            transactions::{
                CommitExecutionDenial, CommitExecutionDenialKind, CommitLog, CommitPhase,
            },
        };
        let mut log = CommitLog::new();
        log.begin_phase(CommitPhase::AuthoritativeMutation);
        let error = TransactionCommitError::Execution {
            denial: CommitExecutionDenial {
                kind: CommitExecutionDenialKind::WorkExhausted,
                partition_identity: Some(23),
            },
            context: ErrorContext::new(
                RelationalSubsystem::Transaction,
                ErrorOperation::ApplyMutation,
            ),
            commit_log: log,
        };
        let failure = crate::domain_computation::WorthQueryProviderSessionFailure::new(
            crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
            crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
            error.detail(),
            crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
        )
        .with_native_preparation_error(error.clone());
        let WorthQueryProviderProgressionOutcome::Denied(denial) = provider_compare_denied(
            crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
                failure,
            ),
        ) else {
            panic!("preparation refusal must stay a denial");
        };
        assert_eq!(denial.kind(), crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialKind::ProviderRejected);
        assert_eq!(denial.stage(), DenialStage::ProviderCommit);
        assert_eq!(denial.detail(), Some(error.detail().as_str()));
        assert_eq!(denial.native_preparation_error(), Some(&error));
    }
}
