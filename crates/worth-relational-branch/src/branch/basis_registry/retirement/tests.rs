//! Exercise real expired-entry pruning and registration replacement between stages.
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::change_source::retained_observation::retirement::RelationalRetainedObservationRetirement;
use crate::change_source::retained_observation::retirement::RelationalRetainedObservationRetirementProgress as Progress;
use crate::runtime::state::subsystems::history::HistorySubsystem;

pub fn assert_original_registry_busy(
    task: &mut RelationalRetainedObservationRetirement,
    history: &HistorySubsystem,
    descriptor: &RelationalBranchBasisDescriptor,
    request: &ExecutionRequest<'_, '_>,
) {
    let registry = &history
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

pub fn assert_pruned_original_registration(
    task: &mut RelationalRetainedObservationRetirement,
    history: &HistorySubsystem,
    descriptor: &RelationalBranchBasisDescriptor,
    request: &ExecutionRequest<'_, '_>,
) {
    let registry = &history
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

pub fn assert_successor_registration_preserved(
    task: &mut RelationalRetainedObservationRetirement,
    history: &HistorySubsystem,
    descriptor: &RelationalBranchBasisDescriptor,
    request: &ExecutionRequest<'_, '_>,
    observe_successor: impl FnOnce() -> AdmittedRelationalBranchBasis,
) {
    let registry = &history
        .branch_cell(descriptor.branch_id())
        .unwrap()
        .basis_registry;
    let key = RelationalRetainedBasisKey::from(descriptor);
    advance_to_expired_entry(task, registry, descriptor, request);
    let original_id = registry.retained.lock().unwrap().entries[&key].registration_id;
    let successor = observe_successor();
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
    assert!(entry.basis.upgrade().unwrap().ptr_eq(&successor.inner));
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
