use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind as AdmissionKind;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind as DemandKind;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind as ExecutionKind;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::ProducerExecutionStop;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::source_readmission::admission_authority_rejection;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::source_readmission::execution_authority_rejection;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::source_readmission::managed_authority_rejection;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::source_readmission::query_admission_denied;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::request_execution::kind as request_execution_kind;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::source_readmission::selected_source_read_denied;
#[test]
fn source_verification_rejects_only_the_original_caller_for_request_stops() {
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind as Attempt;
    use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexDenialCause as Cause;
    use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind as Authorization;
    use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as Refusal;
    for kind in [
        Attempt::Cancelled,
        Attempt::DeadlineExceeded,
        Attempt::DecisionFactBudgetExceeded,
    ] {
        assert!(managed_authority_rejection(Cause::SourceVerification {
            kind,
            authorization: None,
            execution: None
        }));
    }
    assert!(managed_authority_rejection(Cause::SourceVerification {
        kind: Attempt::CurrentAuthorityDenied,
        authorization: Some(Authorization::Refused(Refusal::CapabilityGrantMissing)),
        execution: None,
    }));
    assert!(managed_authority_rejection(Cause::SourceVerification {
        kind: Attempt::DecisionFactBudgetExceeded,
        authorization: None,
        execution: Some(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted
        ),
    }));
    for kind in [Attempt::SourceChanged, Attempt::MissingAuthoritativeFact] {
        assert!(!managed_authority_rejection(Cause::SourceVerification {
            kind,
            authorization: None,
            execution: None
        }));
    }
}
#[test]
fn current_control_stays_caller_local_while_intrinsic_managed_refusals_do_not() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::output_source_observation::SourceQueryObservationStop as Stop;
        use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexDenialCause as Cause;
        use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexRefusal as Refusal;
        use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
        use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture as Posture;
    let original = WorthQueryOutputDemandDenial::new_for_test(
        DemandKind::CurrentControlBusy,
        "current control",
    );
    assert!(managed_authority_rejection(Cause::CurrentControl(
        &original
    )));
    assert!(!managed_authority_rejection(Cause::Refused(
        Refusal::ForeignContext
    )));
    let ProducerExecutionStop::RequestAdmissionDenied(rejected) =
        selected_source_read_denied("producer", Stop::CurrentControl(original))
    else {
        panic!("current control must not fail a shared output");
    };
    let denial = rejected.into_denial_for_test();
    assert_eq!(denial.kind(), DemandKind::CurrentControlBusy);
    assert_eq!(denial.recovery_posture(), Posture::Retryable);
}
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Stop;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::MutationHandlerExecutionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenial;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionDenialKind as Invariant;
use worth_query_execution::facade::domain_computation::WorthQueryOperationProjectionDenialKind as Projection;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Native;

#[test]
fn carried_native_read_stop_survives_producer_admission_and_body_receivers() {
    let expected = DemandKind::ExecutionRequest(Stop::Resource(Resource::WorkExhausted));
    let native = Native::ExecutionResource {
        denial: Resource::WorkExhausted,
        partition_identity: None,
        policy_ancestor: None,
    };
    let admission = WorthQueryApplicationQueryAdmissionDenial::new_for_test(
        AdmissionKind::ExecutionRequest(native),
        "actual source read",
    );
    let ProducerExecutionStop::ExecutionStopped(stopped) =
        query_admission_denied("producer", admission)
    else {
        panic!("resource stop must not be an authority rejection");
    };
    assert_eq!(stopped.kind(), expected);
    assert_eq!(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::projection_budget(Projection::InvariantAdmission(
            Invariant::ExecutionRequest(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted))), Some(expected.clone()));
    let attempt = WorthQueryApplicationAttemptDenial::relational_read_stopped_for_test(
        worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted,
        "body",
    );
    let ProducerExecutionStop::ExecutionStopped(stopped) = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::execution_failed(
            "producer",
            MutationHandlerExecutionDenial::Attempt(attempt),
        ) else {
            panic!("body resource stop remains with execution");
        };
    assert_eq!(stopped.kind(), expected);
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeStop;
    for (cause, expected) in [
        (NativeStop::Cancelled, DemandKind::Cancelled),
        (NativeStop::DeadlineElapsed, DemandKind::TimedOut),
    ] {
        let attempt =
            WorthQueryApplicationAttemptDenial::relational_read_stopped_for_test(cause, "body");
        let ProducerExecutionStop::RequestAdmissionDenied(rejected) =
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::execution_failed(
                    "producer",
                    MutationHandlerExecutionDenial::Attempt(attempt),
                )
            else {
                panic!("body control stop must stay with the request");
            };
        assert_eq!(rejected.into_denial_for_test().kind(), expected);
    }
}

#[test]
fn attributed_native_stop_and_genuine_absent_handler_keep_distinct_causes() {
    let native = Native::ExecutionResource {
        denial: Resource::WorkExhausted,
        partition_identity: Some(17),
        policy_ancestor: Some(2),
    };
    assert_eq!(
        request_execution_kind(native),
        DemandKind::SourceQueryExecution(ExecutionKind::ExecutionRequest(native))
    );
    let ProducerExecutionStop::ExecutionStopped(stopped) = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::execution_failed(
            "producer",
            MutationHandlerExecutionDenial::HandlerNotInstalled,
        ) else {
            panic!("an actually absent handler remains a producer failure");
        };
    assert_eq!(stopped.kind(), DemandKind::ProducerUnavailable);
}
#[test]
fn unpartitioned_kernel_stops_stay_request_local_across_source_and_commit() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_execution_denied;
        use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage as Stage;
        use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome as Outcome;
    for (native, expected) in [
        (
            Native::ExecutionNestedPatternStopped {
                partition_identity: None,
            },
            DemandKind::ExecutionRequest(Stop::NestedStopped),
        ),
        (
            Native::ExecutionWorkerPanicked {
                partition_identity: None,
            },
            DemandKind::ExecutionRequest(Stop::Panicked),
        ),
    ] {
        let source = query_admission_denied(
            "producer",
            WorthQueryApplicationQueryAdmissionDenial::new_for_test(
                AdmissionKind::ExecutionRequest(native),
                "source request executor stopped",
            ),
        )
        .into_output_denial_for_test();
        let commit = provider_session_execution_denied(
            Ok(native),
            Stage::ProviderCommit,
            "commit request executor stopped",
        );
        let Err(commit) = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::outcome::commit_receipt("producer", Outcome::Denied(commit)) else {
                panic!("native commit unexpectedly succeeded");
            };
        assert_eq!(source.kind(), expected);
        assert_eq!(commit.kind(), expected);
        assert_eq!(
            commit.commit_execution_denial(),
            Some((Stage::ProviderCommit, Ok(native)))
        );
    }
}
#[test]
fn managed_authority_diagnostics_keep_the_ordinary_caller_local_boundary() {
    use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind as Auth;
    use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as Refused;
    for kind in [
        AdmissionKind::StalePrincipal,
        AdmissionKind::ForeignPrincipal,
        AdmissionKind::ForeignScope,
        AdmissionKind::StaleScope,
        AdmissionKind::ScopeTypeMismatch,
        AdmissionKind::Authorization(Auth::Refused(Refused::StaleAuthorization)),
    ] {
        assert!(admission_authority_rejection(kind));
    }
    for kind in [
        ExecutionKind::StalePrincipal,
        ExecutionKind::StaleScope,
        ExecutionKind::Authorization(Auth::Refused(Refused::StaleAuthorization)),
    ] {
        assert!(execution_authority_rejection(kind));
    }
    assert!(!admission_authority_rejection(
        AdmissionKind::WorkLimitExceeded
    ));
    assert!(!execution_authority_rejection(
        ExecutionKind::WorkLimitExceeded
    ));
}
