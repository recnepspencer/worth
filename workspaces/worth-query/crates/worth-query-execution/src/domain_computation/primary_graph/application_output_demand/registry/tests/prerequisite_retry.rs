use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture;

#[test]
fn pending_required_prerequisite_preserves_its_retryable_operation() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("dependent", 70, 1);
    let wake = Arc::new(DemandWake {
        _record_capacity: test_record_capacity(),
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            _record_capacity: test_record_capacity(),
            source_commit_capacity: None,
            required_interests: 1,
            wake: Arc::clone(&wake),
            ..record(occurrence(), DemandState::Running, 1)
        },
    );
    let demand_interest = required_interest(&registry, demand_key.clone(), wake);
    let mut denial = WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
        "selected upstream settlement has not entered the managed registry",
    )
    .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable);

    registry.finish_execution_failure(&demand_interest, &mut denial);

    assert_eq!(
        denial.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Retryable
    );
    assert!(matches!(
        registry.state.lock().unwrap().records[&demand_key].state,
        DemandState::Scheduled
    ));
}

/// An execution pinned before its upstream refreshed read the older row, and
/// the refresh retired it before the claim. The claim is stale and retryable:
/// the retry reads the current upstream row.
#[test]
fn an_upstream_retired_after_the_execution_read_it_is_a_retryable_stale_claim() {
    use crate::domain_computation::execution_runtime::source_invalidation::{
        WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
    };
    let registry = WorthQueryOutputDemandRegistry::default();
    let reader_key = key("reader", 71, 1);
    let wake = Arc::new(DemandWake {
        _record_capacity: test_record_capacity(),
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        reader_key.clone(),
        DemandRecord {
            required_interests: 1,
            wake: Arc::clone(&wake),
            ..record(occurrence(), DemandState::Running, 1)
        },
    );
    let reader = required_interest(&registry, reader_key, wake);
    let resources = WorthQueryInvalidationResources::install(
        WorthQueryInvalidationResourceInstallation::bounded(1_000_000, 1 << 20, 1 << 20, 1),
    )
    .unwrap();
    let source_owner =
        crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner::new(
            resources, 1,
        );
    let mut admission = record_admission();
    let context = reader
        .required_context(&source_owner, &mut admission)
        .expect("the running reader issues its required context");
    // The upstream row this execution read is no longer indexed: a newer
    // settlement of its demand retired it.
    let (_lineage, retired) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let graph = world.application.runtime.primary_graph().unwrap();
    let selected = graph.integration_handle().with_runtime(|runtime| {
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        Arc::new(runtime.read_truth().positioned_snapshot(&snapshot).unwrap())
    });
    let consumed = crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence::retained_for_test(
        &source_owner, retired, Arc::from([]), Vec::new(), None, selected,
    );
    let stop = context
        .prepare_prerequisites(std::iter::once(&consumed), &source_owner, &mut admission)
        .err()
        .expect("a retired upstream cannot be claimed");
    assert_eq!(
        stop.kind(),
        WorthQueryOutputDemandDenialKind::PublicationStale
    );
    assert!(
        stop.clone().take_requested_output().is_none(),
        "a retired exact settlement cannot authorize managed readiness recovery"
    );
    assert_eq!(
        stop.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Retryable
    );
}
