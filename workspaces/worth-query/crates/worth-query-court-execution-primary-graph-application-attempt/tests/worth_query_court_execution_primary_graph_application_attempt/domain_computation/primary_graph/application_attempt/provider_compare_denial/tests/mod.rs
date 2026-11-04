use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind as Application;
use worth_query_execution::facade::domain_computation::WorthQueryProviderCompareAndCommitDenial as Compare;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenial as Denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome as Progression;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage as Stage;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_compare_denial::control_stopped_outcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_compare_denied;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_execution_denied;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel as Level;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetDenialKind as Read;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Session;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage;

pub fn denied(compare: Compare) -> Denial {
    let Progression::Denied(denial) = provider_compare_denied(compare) else {
        panic!("pre-effect compare refusal must remain an application denial");
    };
    denial
}

#[test]
fn commit_preparation_keeps_every_execution_kind_typed() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|_active_phase| {
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
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionCommitControlStopped as Stopped;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    for kind in [Control::Cancelled, Control::TimedOut] {
        let Progression::Denied(denial) = control_stopped_outcome(
            Stopped::execution_preparation_for_test(kind, "owner control stop"),
        ) else {
            panic!("preparation control refusal lost HEAD retry route");
        };
        assert_eq!(denial.stage(), Stage::ProviderCommit);
        assert_eq!(denial.execution_denial_cause(), Some(Err(kind)));
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::conditional_operation::application_operation_reentry::sealed_commit::denial::tests::assert_preparation_retry(
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
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    {
        use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::commit_refusals::work_refusal;
        use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated;
        isolated(
            concat!(
                module_path!(),
                "::real_pending_refusal_keeps_idempotency_category_stage_and_evidence"
            ),
            || {
                let error = work_refusal(execution);
                let worth_relational::facade::transactions::TransactionCommitError::Execution {
                    denial,
                    ..
                } = error
                else {
                    panic!("probe must enter through a real execution refusal");
                };
                let worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause) =
                    denial.kind;
                let kind =
                    worth_query_execution::facade::domain_computation::relational_execution_kind(
                        cause,
                        denial.partition_identity,
                    )
                    .unwrap();
                for stage in [Stage::Idempotency, Stage::InvariantExecution] {
                    let application =
                        provider_session_execution_denied(Ok(kind), stage, "the refusing owner");
                    assert_eq!(
                        application.kind(),
                        Application::ExecutionResource {
                            denial: Resource::WorkExhausted,
                            partition_identity: None,
                            policy_ancestor: None
                        }
                    );
                    assert_eq!(application.stage(), stage);
                    assert_eq!(
                        application.execution_denial_cause(),
                        Some(Ok(Session::ExecutionResource {
                            denial: Resource::WorkExhausted,
                            partition_identity: None,
                            policy_ancestor: None,
                        }))
                    );
                }
            },
        );
    }
}

#[test]
fn nonallocation_session_evidence_preserves_the_mapped_detail_and_read_set_stage() {
    let failure = WorthQueryProviderSessionFailure::new(
        Session::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::Commit,
        "native session detail",
        Default::default(),
    );
    let application =
        Denial::provider_rejected_with_detail_for_test(Stage::ProviderCommit, "mapped detail")
            .with_provider_session_failure_for_test(failure);
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
    use worth_relational::facade::errors::ErrorContext;
    use worth_relational::facade::errors::ErrorOperation;
    use worth_relational::facade::errors::RelationalSubsystem;
    use worth_relational::facade::mvcc::TransactionCommitError;
    use worth_relational::facade::transactions::CommitExecutionDenial;
    use worth_relational::facade::transactions::CommitExecutionDenialKind;
    use worth_relational::facade::transactions::CommitLog;
    use worth_relational::facade::transactions::CommitPhase;
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
    let failure = worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure::new(
        worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
        worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
        error.detail(),
        worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
    .with_native_preparation_error_for_test(error.clone());
    let Progression::Denied(denial) = provider_compare_denied(
        worth_query_execution::facade::domain_computation::WorthQueryProviderCompareAndCommitDenial::ProviderSession(
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
    let earlier = provider_session_execution_denied(Ok(cause), Stage::Idempotency, error.detail())
        .with_provider_session_failure_for_test(failure);
    assert_eq!(earlier.kind(), denial.kind());
    assert_eq!(
        earlier.execution_denial_cause(),
        denial.execution_denial_cause()
    );
    assert_eq!(earlier.stage(), Stage::Idempotency);
    assert_eq!(earlier.native_preparation_error(), Some(&error));
    let invariant_failure = worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionFailure::new(
        worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind::ExecutionDenied(cause),
        error.detail(),
    );
    let invariant = Denial::invariant_execution_denied_for_test(
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

#[test]
fn read_fold_checked_session_read_stop_is_the_commit_denial_cause() {
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure as Failure;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage as Protocol;
    for stop in [Control::Cancelled, Control::TimedOut] {
        let failure = Failure::checked_read_stopped_for_test(
            Err(stop),
            Protocol::Commit,
            "native read stopped",
        );
        let denial = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied(failure);
        assert_eq!(denial.execution_denial_cause(), Some(Err(stop)));
        assert_eq!(denial.stage(), Stage::ProviderCommit);
    }
}
