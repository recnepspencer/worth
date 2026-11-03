use graph::WorthQueryApplicationCommitDenialKind;
use graph::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::outcome::commit_receipt;
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::execution::denial::denial;
use worth_query_execution::domain_computation::primary_graph as graph;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionCommitDeferred;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionCommitDeferredKind;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
use worth_relational::facade::mvcc::RelationalPublicationDeferred;

pub fn stopped(stop: Stop) -> WorthQueryOutputDemandDenial {
    let deferred = WorthQueryProviderSessionCommitDeferred::new(
        WorthQueryProviderSessionCommitDeferredKind::RelationalDeferred(
            RelationalPublicationDeferred::CompanionPreflight(stop),
        ),
        "",
    );
    commit_receipt(
        "producer",
        WorthQueryApplicationCommitOutcome::Deferred(
            worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDeferred::from_provider_session(deferred),
        ),
    )
    .unwrap_err()
}

#[test]
fn a_commit_a_companion_has_no_room_for_stops_for_that_budget() {
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture;
    use WorthQueryOutputDemandDenialKind as Kind;
    for (stop, expected_kind) in [
        (
            Stop::PreparationMemoryCounterOverflow,
            Kind::RetentionBudgetExceeded,
        ),
        /* ee07da2b0e retired preflight forecast-work exhaustion; actual request stops use Execution. */
        (Stop::WorkCounterOverflow, Kind::WorkBudgetExceeded),
    ] {
        let observed = stopped(stop);
        assert_eq!(observed.kind(), expected_kind);
        assert_eq!(
            observed.recovery_posture(),
            WorthQueryOutputDemandRecoveryPosture::Terminal
        );
        assert_eq!(observed.subject(), "producer");
        assert_eq!(observed.publication_companion_stop(), Some(stop));
        assert_eq!(observed.producer_admission_stop(), None);
        let diagnostic = observed.clone();
        assert_eq!(diagnostic.publication_companion_stop(), Some(stop));
        assert_eq!(diagnostic, observed);
    }
    // A companion that is not ready is no budget of the advance.
    let pending = stopped(Stop::TopologyPending);
    assert_eq!(pending.kind(), Kind::ProducerUnavailable);
    assert_eq!(pending.publication_companion_stop(), None);
}

#[test]
fn every_execution_commit_kind_survives_the_producer_boundary() {
    use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    use WorthQueryApplicationCommitDenialKind as Application;
    for (session, expected, expected_demand) in [
        (
            Session::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: Some(7),
                policy_ancestor: Some(2),
            },
            Application::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: Some(7),
                policy_ancestor: Some(2),
            },
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        ),
        (
            Session::ExecutionNestedPatternStopped {
                partition_identity: Some(7),
            },
            Application::ExecutionNestedPatternStopped {
                partition_identity: Some(7),
            },
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        ),
        (
            Session::ExecutionWorkerPanicked {
                partition_identity: Some(7),
            },
            Application::ExecutionWorkerPanicked {
                partition_identity: Some(7),
            },
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        ),
        (
            Session::ExecutionWorkerPanicked {
                partition_identity: None,
            },
            Application::ExecutionWorkerPanicked {
                partition_identity: None,
            },
            WorthQueryOutputDemandDenialKind::ExecutionRequest(
                worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Panicked,
            ),
        ),
        (
            Session::ExecutionUncheckedCustomKernel {
                partition_identity: Some(7),
            },
            Application::ExecutionUncheckedCustomKernel {
                partition_identity: Some(7),
            },
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        ),
        (
            Session::ExecutionIdentitiesNotCanonical {
                partition_identity: Some(7),
            },
            Application::ExecutionIdentitiesNotCanonical {
                partition_identity: Some(7),
            },
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        ),
    ] {
        let commit = worth_query_execution::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied(
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure::new(session,
                worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                "producer commit preparation refused", Default::default()),
        );
        let observed = commit_receipt(
            "producer",
            WorthQueryApplicationCommitOutcome::Denied(commit),
        )
        .unwrap_err();
        assert_eq!(observed.kind(), expected_demand);
        assert_eq!(observed.commit_denial_kind(), Some(expected));
    }
}

#[test]
fn pending_execution_evidence_keeps_heads_producer_category() {
    use worth_query_execution::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_execution_denied;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage as Stage;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    let kind = Session::ExecutionWorkerPanicked {
        partition_identity: Some(7),
    };
    let commit = provider_session_execution_denied(
        Ok(kind),
        Stage::Idempotency,
        "pending publication refused",
    );
    let observed = commit_receipt(
        "producer",
        WorthQueryApplicationCommitOutcome::Denied(commit),
    )
    .unwrap_err();
    assert_eq!(
        observed.kind(),
        WorthQueryOutputDemandDenialKind::ProducerUnavailable
    );
    assert_eq!(observed.commit_denial_kind(), Some(worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind::ExecutionWorkerPanicked { partition_identity: Some(7) }));
    assert_eq!(
        observed.commit_execution_denial(),
        Some((Stage::Idempotency, Ok(kind)))
    );
}

#[test]
fn root_request_work_stop_is_distinct_from_attributed_producer_work() {
    use worth_query_execution::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_execution_denied;
    use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Request;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage as Stage;
    use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    use WorthQueryOutputDemandDenialKind as Demand;
    for (cause, expected) in [
        (
            Ok(Session::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: None,
            }),
            Demand::ExecutionRequest(Request::Resource(Resource::WorkExhausted)),
        ),
        (
            Ok(Session::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: Some(7),
                policy_ancestor: None,
            }),
            Demand::ProducerUnavailable,
        ),
        (
            Ok(Session::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: Some(2),
            }),
            Demand::ProducerUnavailable,
        ),
        (Err(Control::Cancelled), Demand::Cancelled),
        (Err(Control::TimedOut), Demand::TimedOut),
    ] {
        let commit = provider_session_execution_denied(
            cause,
            Stage::ProviderCommit,
            "request stopped before publication",
        );
        let observed = commit_receipt(
            "producer",
            WorthQueryApplicationCommitOutcome::Denied(commit),
        )
        .unwrap_err();
        assert_eq!(observed.kind(), expected);
        assert_eq!(
            observed.commit_execution_denial(),
            Some((Stage::ProviderCommit, cause))
        );
        let expected_commit = match cause {
            Ok(Session::ExecutionResource {
                denial,
                partition_identity,
                policy_ancestor,
            }) => WorthQueryApplicationCommitDenialKind::ExecutionResource {
                denial,
                partition_identity,
                policy_ancestor,
            },
            Err(_) => WorthQueryApplicationCommitDenialKind::ProviderRejected,
            _ => unreachable!(),
        };
        assert_eq!(observed.commit_denial_kind(), Some(expected_commit));
    }
}

#[test]
fn consumed_verification_capacity_reaches_the_original_demand_denial() {
    use worth_query_execution::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied;
    use worth_query_execution::facade::domain_computation::VerificationCapacityStop as Capacity;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind as Kind;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage as Stage;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as ProviderKind;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage;
    for capacity in [Some(Capacity::ScratchAllocationRefused { bytes: 65 }), None] {
        let mut failure = WorthQueryProviderSessionFailure::new(
            ProviderKind::ProviderRejected,
            WorthQueryProviderSessionProtocolStage::Commit,
            "consumed output currentness exceeded publication capacity",
            Default::default(),
        );
        if let Some(stop) = capacity {
            failure = failure.with_output_verification_capacity_stop(stop);
        }
        let commit = provider_session_denied(failure);
        assert_eq!(commit.stage(), Stage::ProviderCommit);
        let denial = commit_receipt(
            "preserve-producer",
            WorthQueryApplicationCommitOutcome::Denied(commit),
        )
        .unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::ProducerUnavailable
        );
        assert_eq!(denial.commit_denial_kind(), Some(Kind::ProviderRejected));
        assert_eq!(
            denial.recovery_posture(),
            worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture::Terminal
        );
        assert_eq!(denial.output_verification_capacity_stop(), capacity);
        assert_eq!(denial.publication_companion_stop(), None);
        assert_eq!(denial.producer_admission_stop(), None);
        assert_eq!(denial.clone().output_verification_capacity_stop(), capacity);
    }
}
