//! A closed real Home retains its seal's original charge until explicit reclaim.
use super::*;

#[test]
fn closed_home_keeps_transition_admission_funded_until_original_world_reclamation() {
    let directory = tempfile::tempdir().unwrap();
    let budget = worth_execution::SerialMemoryBudget::new(None);
    let cancellation = worth_execution::CancellationSource::new();
    let request =
        worth_execution::SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    drop(fixture.bootstrap_root(&execution));
    let home = fixture.home;
    let observation = home.transition_admission_observation();
    let charged = observation.held_memory_bytes();
    assert!(
        charged > 0,
        "the accounted original request funded the real seal"
    );
    let sealed = home.seal_world(&execution).unwrap_or_else(|refused| {
        panic!("the actual quiescent World seals: {:?}", refused.reason())
    });
    let retired = match sealed.finish_close(&execution) {
        ApplicationHomeCloseOutcome::Closed { allocation, .. } => allocation,
        ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
            panic!("the actual Home must close: {reason:?}")
        }
    };
    assert_eq!(
        observation.held_memory_bytes(),
        charged,
        "the retired original World still owns its attached seal verifier"
    );
    let held_before_reclaim = budget.charged_memory_bytes();
    assert!(held_before_reclaim >= charged);
    retired.reclaim();
    assert_eq!(
        observation.held_memory_bytes(),
        0,
        "the read-only observation cannot retain the seal or its funding"
    );
    assert!(
        held_before_reclaim - budget.charged_memory_bytes() >= charged,
        "reclaim returns the original seal's actual budget reservation"
    );
}
