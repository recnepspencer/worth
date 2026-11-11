//! Exercise real expired-entry pruning and registration replacement between stages.
use super::*;
use crate::change_source::{
    RelationalRetainedObservationRetirement,
    RelationalRetainedObservationRetirementProgress as Progress,
};
use crate::runtime::RelationalRuntime;

pub(crate) fn assert_original_registry_busy(
    task: &mut RelationalRetainedObservationRetirement,
    runtime: &RelationalRuntime,
    descriptor: &RelationalBranchBasisDescriptor,
    request: &ExecutionRequest<'_, '_>,
) {
    let registry = &runtime
        .history
        .branch_cell(descriptor.branch_id())
        .unwrap()
        .basis_registry;
    let key = RelationalRetainedBasisKey::from(descriptor);
    let gate = registry.retained.lock().unwrap();
    let registration_id = gate.entries[&key].registration_id;
    let metrics = registry.metrics.snapshot();
    let mut busy = false;
    for _ in 0..12 {
        match task.advance(request) {
            Progress::Advanced => {}
            Progress::Busy => {
                busy = true;
                break;
            }
            result => panic!("unexpected retirement result: {result:?}"),
        }
    }
    assert!(
        busy,
        "the original registry gate must prevent exact removal"
    );
    assert_eq!(gate.entries[&key].registration_id, registration_id);
    assert!(gate.entries[&key].basis.upgrade().is_none());
    assert_eq!(registry.metrics.snapshot(), metrics);
    assert_eq!(task.advance(request), Progress::Busy);
    drop(gate);
    finish(task, request);
    assert!(!registry.retained.lock().unwrap().entries.contains_key(&key));
    assert_eq!(registry.metrics.snapshot().0, metrics.0 - 1);
    let after = registry.metrics.snapshot();
    assert_eq!(task.advance(request), Progress::Complete);
    assert_eq!(registry.metrics.snapshot(), after);
}

pub(crate) fn assert_pruned_original_registration(
    task: &mut RelationalRetainedObservationRetirement,
    runtime: &RelationalRuntime,
    descriptor: &RelationalBranchBasisDescriptor,
    request: &ExecutionRequest<'_, '_>,
) {
    let registry = &runtime
        .history
        .branch_cell(descriptor.branch_id())
        .unwrap()
        .basis_registry;
    advance_to_expired_entry(task, registry, descriptor, request);
    let before = registry.metrics.snapshot();
    assert!(registry.readmit_retained(descriptor).is_none());
    assert_eq!(registry.metrics.snapshot().0, before.0 - 1);
    let pruned = registry.metrics.snapshot();
    finish(task, request);
    assert_eq!(registry.metrics.snapshot().0, pruned.0);
    assert_eq!(registry.metrics.snapshot().2, pruned.2);
}

pub(crate) fn assert_successor_registration_preserved(
    task: &mut RelationalRetainedObservationRetirement,
    runtime: &RelationalRuntime,
    descriptor: &RelationalBranchBasisDescriptor,
    request: &ExecutionRequest<'_, '_>,
) {
    let registry = &runtime
        .history
        .branch_cell(descriptor.branch_id())
        .unwrap()
        .basis_registry;
    let key = RelationalRetainedBasisKey::from(descriptor);
    advance_to_expired_entry(task, registry, descriptor, request);
    let original_id = registry.retained.lock().unwrap().entries[&key].registration_id;
    let (_, successor) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let successor_id = registry.retained.lock().unwrap().entries[&key].registration_id;
    assert_ne!(successor_id, original_id);
    assert_eq!(
        registry.metrics.snapshot().0,
        registry.retained.lock().unwrap().entries.len() as u64
    );
    let before = registry.metrics.snapshot();
    finish(task, request);
    let state = registry.retained.lock().unwrap();
    let entry = &state.entries[&key];
    assert_eq!(entry.registration_id, successor_id);
    assert!(Arc::ptr_eq(
        &entry.basis.upgrade().unwrap(),
        &successor.inner
    ));
    assert_eq!(registry.metrics.snapshot().0, before.0);
    assert_eq!(registry.metrics.snapshot().2, before.2);
    assert_eq!(registry.metrics.snapshot().0, state.entries.len() as u64);
    drop(state);
    drop(successor);
    assert!(registry.retained.lock().unwrap().entries.is_empty());
    assert_eq!(registry.metrics.snapshot().0, 0);
}

fn advance_to_expired_entry(
    task: &mut RelationalRetainedObservationRetirement,
    registry: &RelationalBranchBasisRegistry,
    descriptor: &RelationalBranchBasisDescriptor,
    request: &ExecutionRequest<'_, '_>,
) {
    let key = RelationalRetainedBasisKey::from(descriptor);
    for _ in 0..12 {
        let expired = registry.retained.lock().unwrap().entries[&key]
            .basis
            .upgrade()
            .is_none();
        if expired {
            return;
        }
        assert_eq!(task.advance(request), Progress::Advanced);
    }
    panic!("the two original shares must hand off their unique inner");
}

fn finish(task: &mut RelationalRetainedObservationRetirement, request: &ExecutionRequest<'_, '_>) {
    for _ in 0..12 {
        match task.advance(request) {
            Progress::Advanced => {}
            Progress::Complete => return,
            result => panic!("unexpected retirement result: {result:?}"),
        }
    }
    panic!("retirement did not reach its terminal state");
}
