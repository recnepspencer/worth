use super::super::WorthQueryApplicationCommitDenialKind as Application;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use crate::domain_computation::{
    WorthQueryDecisionReadSetDenialKind as Read, WorthQueryDecisionReadSetFailure,
    WorthQueryProviderSessionDenialKind as Session, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolStage,
};

fn denied(compare: Compare) -> Denial {
    let Progression::Denied(denial) = provider_compare_denied(compare) else {
        panic!("pre-effect compare refusal must remain an application denial");
    };
    denial
}

#[test]
fn commit_preparation_keeps_every_execution_kind_typed() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        let mut kinds = vec![
            (
                Session::ExecutionNestedPatternStopped {
                    partition_identity: Some(7),
                },
                Application::ExecutionNestedPatternStopped {
                    partition_identity: Some(7),
                },
            ),
            (
                Session::ExecutionWorkerPanicked {
                    partition_identity: Some(7),
                },
                Application::ExecutionWorkerPanicked {
                    partition_identity: Some(7),
                },
            ),
            (
                Session::ExecutionUncheckedCustomKernel {
                    partition_identity: Some(7),
                },
                Application::ExecutionUncheckedCustomKernel {
                    partition_identity: Some(7),
                },
            ),
            (
                Session::ExecutionIdentitiesNotCanonical {
                    partition_identity: Some(7),
                },
                Application::ExecutionIdentitiesNotCanonical {
                    partition_identity: Some(7),
                },
            ),
        ];
        for resource in [
            Resource::WorkExhausted,
            Resource::WorkCounterOverflow,
            Resource::RetainedBytesExhausted,
            Resource::ScratchCapacityExceeded,
            Resource::ResultCapacityExceeded,
            Resource::CapacityOverflow,
            Resource::ChargedBytesOverflow,
            Resource::WorkerLimit,
            Resource::PolicyMemoryLimit,
            Resource::WorkLimit,
            Resource::NestedLeaseMisuse,
            Resource::NoActiveExecutionScope,
            Resource::EquivalenceContractUnavailable,
            Resource::MemoryLimit {
                requested: 31,
                admitted: 17,
                level: Level::Policy,
            },
            Resource::MemoryLimit {
                requested: 31,
                admitted: 17,
                level: Level::Process,
            },
            Resource::MemoryLimit {
                requested: 31,
                admitted: 17,
                level: Level::Declared,
            },
        ] {
            kinds.push((
                Session::ExecutionResource {
                    denial: resource,
                    partition_identity: Some(7),
                    policy_ancestor: None,
                },
                Application::ExecutionResource {
                    denial: resource,
                    partition_identity: Some(7),
                    policy_ancestor: None,
                },
            ));
        }
        for (kind, expected) in kinds {
            let failure = WorthQueryProviderSessionFailure::new(
                kind,
                WorthQueryProviderSessionProtocolStage::Commit,
                "owner evidence",
                Default::default(),
            );
            let application = denied(Compare::ProviderSession(failure));
            assert_eq!(
                application.kind(),
                expected,
                "execution category folded: {kind:?}"
            );
            assert_eq!(application.stage(), Stage::ProviderCommit);
        }
    });
}

#[test]
fn existing_maintenance_and_read_capacity_outcomes_are_preserved() {
    for (kind, expected) in [
        (
            Session::IndexMaintenanceBudgetExceeded,
            Application::IndexMaintenanceBudgetExceeded,
        ),
        (
            Session::IndexGenerationIdentityExhausted,
            Application::IndexGenerationIdentityExhausted,
        ),
    ] {
        let failure = WorthQueryProviderSessionFailure::new(
            kind,
            WorthQueryProviderSessionProtocolStage::Commit,
            "index owner",
            Default::default(),
        );
        assert_eq!(denied(Compare::ProviderSession(failure)).kind(), expected);
    }
    for (kind, expected) in [
        (
            Read::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 3,
            },
            Application::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 3,
            },
        ),
        (
            Read::RetentionCapacityExhausted,
            Application::RetentionCapacityExhausted,
        ),
        (
            Read::RetentionIdentityExhausted,
            Application::RetentionIdentityExhausted,
        ),
        (
            Read::SnapshotIdentityExhausted,
            Application::SnapshotIdentityExhausted,
        ),
    ] {
        assert_eq!(
            denied(Compare::DecisionReadSet(
                WorthQueryDecisionReadSetFailure::new(kind, "read owner")
            ))
            .kind(),
            expected
        );
    }
}

#[test]
fn preparation_control_refusal_and_ordinary_interruption_keep_different_head_postures() {
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped as Stopped,
        WorthQueryProviderSessionControlStopKind as Control,
    };
    for kind in [Control::Cancelled, Control::TimedOut] {
        let Progression::Denied(denial) =
            control_stopped_outcome(Stopped::execution_preparation(kind, "owner control stop"))
        else {
            panic!("preparation control refusal lost HEAD retry route");
        };
        assert_eq!(denial.stage(), Stage::ProviderCommit);
        assert_eq!(denial.execution_denial_cause(), Some(Err(kind)));
        crate::domain_computation::primary_graph::conditional_operation::assert_preparation_retry(
            &denial,
        );
        match (
            kind,
            control_stopped_outcome(Stopped::new(kind, "ordinary interruption")),
        ) {
            (Control::Cancelled, Progression::Cancelled)
            | (Control::TimedOut, Progression::TimedOut) => {}
            _ => panic!("ordinary operation interruption changed its HEAD meaning"),
        }
    }
}

#[test]
fn real_pending_refusal_keeps_idempotency_category_stage_and_evidence() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        use crate::domain_computation::primary_graph::bootstrap_publication::{
            commit_refusals::work_refusal, execution_refusals::isolated,
        };
        isolated(
            concat!(
                module_path!(),
                "::real_pending_refusal_keeps_idempotency_category_stage_and_evidence"
            ),
            || {
                let error = work_refusal();
                let worth_relational::facade::transactions::TransactionCommitError::Execution {
                    denial,
                    ..
                } = error
                else {
                    panic!("probe must enter through a real execution refusal");
                };
                let worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause) =
                    denial.kind;
                let kind = crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause, denial.partition_identity).unwrap();
                for stage in [Stage::Idempotency, Stage::InvariantExecution] {
                    let application =
                        provider_session_kind_denied(kind, stage, "the refusing owner");
                    assert_eq!(
                        application.kind(),
                        Application::ExecutionResource {
                            denial: Resource::WorkExhausted,
                            partition_identity: Some(1),
                            policy_ancestor: None
                        }
                    );
                    assert_eq!(application.stage(), stage);
                    assert_eq!(
                        application.execution_denial_cause(),
                        Some(Ok(Session::ExecutionResource {
                            denial: Resource::WorkExhausted,
                            partition_identity: Some(1),
                            policy_ancestor: None,
                        }))
                    );
                }
            },
        );
    });
}

#[test]
fn nonallocation_session_evidence_preserves_the_mapped_detail_and_read_set_stage() {
    let failure = WorthQueryProviderSessionFailure::new(
        Session::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::Commit,
        "native session detail",
        Default::default(),
    );
    let application = Denial::provider_rejected_with_detail(Stage::ProviderCommit, "mapped detail")
        .with_provider_session_failure(failure);
    assert_eq!(application.detail(), Some("mapped detail"));
    assert!(application.allocation_denial().is_none());
    let read = denied(Compare::DecisionReadSet(
        WorthQueryDecisionReadSetFailure::new(Read::SnapshotIdentityExhausted, "read owner"),
    ));
    assert_eq!(read.stage(), Stage::ProviderCommit);
    assert_eq!(read.kind(), Application::SnapshotIdentityExhausted);
}

#[test]
fn native_cause_context_and_commit_log_survive_application_kind_mapping() {
    use worth_relational::facade::{
        errors::{ErrorContext, ErrorOperation, RelationalSubsystem},
        mvcc::TransactionCommitError,
        transactions::{CommitExecutionDenial, CommitExecutionDenialKind, CommitLog, CommitPhase},
    };
    let mut log = CommitLog::new();
    log.begin_phase(CommitPhase::AuthoritativeMutation);
    let error = TransactionCommitError::Execution {
        denial: CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted),
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
    let Progression::Denied(denial) = provider_compare_denied(
        crate::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
            failure.clone(),
        ),
    ) else {
        panic!("preparation refusal must stay a denial");
    };
    assert_eq!(
        denial.kind(),
        Application::ExecutionResource {
            denial: Resource::WorkExhausted,
            partition_identity: Some(23),
            policy_ancestor: None,
        }
    );
    let cause = Session::ExecutionResource {
        denial: Resource::WorkExhausted,
        partition_identity: Some(23),
        policy_ancestor: None,
    };
    assert_eq!(denial.execution_denial_cause(), Some(Ok(cause)));
    let earlier = provider_session_kind_denied(cause, Stage::Idempotency, error.detail())
        .with_provider_session_failure(failure);
    assert_eq!(earlier.kind(), denial.kind());
    assert_eq!(
        earlier.execution_denial_cause(),
        denial.execution_denial_cause()
    );
    assert_eq!(earlier.stage(), Stage::Idempotency);
    assert_eq!(earlier.native_preparation_error(), Some(&error));
    let invariant_failure = crate::domain_computation::WorthQueryInvariantExecutionFailure::new(
        crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ExecutionDenied(cause),
        error.detail(),
    );
    let invariant = super::Denial::invariant_execution_denied(
        Stage::InvariantExecution,
        invariant_failure.clone(),
    );
    assert_eq!(invariant.kind(), denial.kind());
    assert_eq!(
        invariant.execution_denial_cause(),
        denial.execution_denial_cause()
    );
    assert_eq!(
        invariant.invariant_execution_failure(),
        Some(&invariant_failure)
    );
    assert_eq!(denial.stage(), Stage::ProviderCommit);
    assert_eq!(denial.detail(), Some(error.detail().as_str()));
    assert_eq!(denial.native_preparation_error(), Some(&error));
}
