//! Genuine owner admission and original close-task metadata retention.
use super::*;
use crate::lifecycle::{RuntimeWorldOwnerLifecycleObservation, RuntimeWorldOwnerRoot};
use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest, WorkCeilingDenial};

struct Clock;
impl crate::lifecycle::RuntimeWorldClockSource for Clock {
    fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
        crate::lifecycle::RuntimeWorldInstant::from_ticks(0)
    }
}
fn owner() -> (
    crate::branch::reference_test_fixture::RealReferenceFixture,
    RuntimeWorldOwnerRoot<(), (), (), (), ()>,
) {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(8, 8);
    let owner = RuntimeWorldOwnerRoot::new(fixture.owner_inputs(
        crate::budget::RuntimeWorldBudgets::without_policy_limits(),
        crate::lifecycle::RuntimeWorldClock::from_source(Clock),
    ))
    .unwrap();
    (fixture, owner)
}

#[test]
fn preclose_observation_uses_original_gates_without_allocating_a_close_task() {
    let (_fixture, owner) = owner();
    let memory = SerialMemoryBudget::new(Some(0));
    let policy = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&policy);
    owner.try_preflight_close_admission(&request).unwrap();
    assert_eq!(memory.charged_memory_bytes(), 0);
    assert!(owner.state.close.lock().unwrap().progress.is_none());
    assert!(owner.state.bootstrap.try_lock().is_ok());
    assert!(owner.state.operation.state.try_lock().is_ok());
    assert!(matches!(
        owner.try_prepare_close_admission(&request),
        Err(Denial::Execution(WorkCeilingDenial::Admission(_)))
    ));
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );
    assert!(owner.state.close.lock().unwrap().progress.is_none());
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn prepared_close_task_retains_original_metadata_grant_until_owner_release() {
    let (_fixture, owner) = owner();
    let memory = SerialMemoryBudget::new(Some(1 << 20));
    let policy = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&policy);
    owner.try_preflight_close_admission(&request).unwrap();
    assert_eq!(memory.charged_memory_bytes(), 0);
    let admission = owner.try_prepare_close_admission(&request).unwrap();
    assert!(memory.charged_memory_bytes() > 0);
    drop(admission);
    assert_eq!(memory.charged_memory_bytes(), 0);
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );

    let admission = owner.try_prepare_close_admission(&request).unwrap();
    let task = admission.task.clone();
    let retained = memory.charged_memory_bytes();
    admission.begin();
    assert!(Arc::ptr_eq(
        owner.state.close.lock().unwrap().progress.as_ref().unwrap(),
        &task
    ));
    assert_eq!(memory.charged_memory_bytes(), retained);
    drop(owner);
    assert_eq!(memory.charged_memory_bytes(), retained);
    drop(task);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
