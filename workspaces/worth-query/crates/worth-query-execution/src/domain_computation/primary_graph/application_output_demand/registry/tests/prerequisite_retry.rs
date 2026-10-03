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
