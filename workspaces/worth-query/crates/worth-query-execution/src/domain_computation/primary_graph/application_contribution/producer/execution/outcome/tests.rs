use super::*;
use crate::domain_computation::provider_session::{
    WorthQueryProviderSessionCommitDeferred, WorthQueryProviderSessionCommitDeferredKind,
};
use worth_relational::facade::mvcc::{
    CompanionPreflightStop as Stop, RelationalPublicationDeferred,
};

fn stopped(stop: Stop) -> WorthQueryOutputDemandDenialKind {
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
    .kind()
}

#[test]
fn a_commit_a_companion_has_no_room_for_stops_for_that_budget() {
    use WorthQueryOutputDemandDenialKind as Kind;
    let retained = Stop::RetainedCompanionCapacityExhausted {
        requested: 2,
        retained: 1,
        maximum: 2,
    };
    assert_eq!(stopped(retained), Kind::RetentionBudgetExceeded);
    let prepared = Stop::PreparationMemoryExhausted {
        required: 2,
        maximum: 1,
    };
    assert_eq!(stopped(prepared), Kind::RetentionBudgetExceeded);
    let work = Stop::WorkExhausted {
        required: 2,
        maximum: 1,
    };
    assert_eq!(stopped(work), Kind::WorkBudgetExceeded);
    // A companion that is not ready is no budget of the advance.
    assert_eq!(stopped(Stop::TopologyPending), Kind::ProducerUnavailable);
}

#[test]
fn every_execution_commit_kind_survives_the_producer_boundary() {
    use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    use WorthQueryApplicationCommitDenialKind as Application;
    for (session, expected) in [
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
        ),
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
        assert_eq!(
            observed.kind(),
            WorthQueryOutputDemandDenialKind::ProducerUnavailable
        );
        assert_eq!(observed.commit_denial_kind(), Some(expected));
    }
}

#[test]
fn pending_execution_evidence_keeps_heads_producer_category() {
    use crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_kind_denied;
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage as Stage;
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    let kind = Session::ExecutionWorkerPanicked {
        partition_identity: Some(7),
    };
    let commit =
        provider_session_kind_denied(kind, Stage::Idempotency, "pending publication refused");
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
