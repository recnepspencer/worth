use super::*;
use crate::domain_computation::provider_session::{
    WorthQueryProviderSessionCommitDeferred, WorthQueryProviderSessionCommitDeferredKind,
};
use worth_relational::facade::mvcc::{
    CompanionPreflightStop as Stop, RelationalPublicationDeferred,
};

fn stopped(stop: Stop) -> WorthQueryOutputDemandDenial {
    let deferred = WorthQueryProviderSessionCommitDeferred::new(
        WorthQueryProviderSessionCommitDeferredKind::RelationalDeferred(
            RelationalPublicationDeferred::CompanionPreflight(stop),
        ),
        "",
    );
    commit_receipt(
        "producer",
        WorthQueryApplicationCommitOutcome::Deferred(
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferred::from_provider_session(deferred),
        ),
    )
    .unwrap_err()
}

#[test]
fn a_commit_a_companion_has_no_room_for_stops_for_that_budget() {
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture;
    use WorthQueryOutputDemandDenialKind as Kind;
    for (stop, expected_kind) in [
        (
            Stop::RetainedCompanionCapacityExhausted {
                requested: 2,
                retained: 1,
                maximum: 2,
            },
            Kind::RetentionBudgetExceeded,
        ),
        (
            Stop::PreparationMemoryExhausted {
                required: 2,
                maximum: 1,
            },
            Kind::RetentionBudgetExceeded,
        ),
        (
            Stop::PreparationMemoryCounterOverflow,
            Kind::RetentionBudgetExceeded,
        ),
        (
            Stop::WorkExhausted {
                required: 2,
                maximum: 1,
            },
            Kind::WorkBudgetExceeded,
        ),
        (
            Stop::WorkExhaustedWithoutRequirement { maximum: 1 },
            Kind::WorkBudgetExceeded,
        ),
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
    use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
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
                graph::WorthQueryAdvancementDenial::Panicked,
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
        let commit = crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied(
            crate::domain_computation::WorthQueryProviderSessionFailure::new(session,
                crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
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
    use crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_execution_denied;
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage as Stage;
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
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
    assert_eq!(observed.commit_denial_kind(), Some(crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::ExecutionWorkerPanicked { partition_identity: Some(7) }));
    assert_eq!(
        observed.commit_execution_denial(),
        Some((Stage::Idempotency, Ok(kind)))
    );
}

#[test]
fn root_request_work_stop_is_distinct_from_attributed_producer_work() {
    use crate::domain_computation::primary_graph::{
        application_attempt::provider_compare_denial::provider_session_execution_denied,
        WorthQueryAdvancementDenial as Request, WorthQueryApplicationCommitDenialStage as Stage,
        WorthQueryManagedComputationResourceDenial as Resource,
    };
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
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
    use crate::domain_computation::primary_graph::{
        application_attempt::provider_compare_denial::provider_session_denied,
        VerificationCapacityStop as Capacity, WorthQueryApplicationCommitDenialKind as Kind,
        WorthQueryApplicationCommitDenialStage as Stage,
    };
    use crate::domain_computation::{
        WorthQueryProviderSessionDenialKind as ProviderKind, WorthQueryProviderSessionFailure,
        WorthQueryProviderSessionProtocolStage,
    };
    for capacity in [
        Some(Capacity::Meter(Stop::PreparationMemoryExhausted {
            required: 65,
            maximum: 64,
        })),
        Some(Capacity::ScratchAllocationRefused { bytes: 65 }),
        None,
    ] {
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
            graph::WorthQueryOutputDemandRecoveryPosture::Terminal
        );
        assert_eq!(denial.output_verification_capacity_stop(), capacity);
        assert_eq!(denial.publication_companion_stop(), None);
        assert_eq!(denial.producer_admission_stop(), None);
        assert_eq!(denial.clone().output_verification_capacity_stop(), capacity);
    }
}
