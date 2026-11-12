//! Real index assertions using only the court's original source and task.
use super::*;

pub(in crate::relational_source) fn assert_observation_busy_and_indexes(
    task: &mut RelationalBridgeObservationRetirement,
    source: &RuntimeBridgeRelationalSource,
    request: &ExecutionRequest<'_, '_>,
    expected_remaining: usize,
) {
    let snapshot = task.lease.snapshot_id;
    let gate = source.observation_bindings.entries.lock().unwrap();
    let commit = gate.by_snapshot[&snapshot].commit_id.unwrap();
    let original = gate.by_snapshot[&snapshot].retained.snapshot_id();
    let original_count = gate.by_commit[&commit].len();
    assert_eq!(task.advance(request, source), Progress::Busy);
    assert_eq!(gate.by_snapshot[&snapshot].retained.snapshot_id(), original);
    assert_eq!(gate.by_commit[&commit].len(), original_count);
    drop(gate);
    assert_eq!(task.advance(request, source), Progress::Advanced);
    let index = source.observation_bindings.entries.lock().unwrap();
    assert!(!index.by_snapshot.contains_key(&snapshot));
    if expected_remaining == 0 {
        assert!(!index.by_commit.contains_key(&commit));
        assert_eq!(task._removed_commit.as_ref().unwrap().0, commit);
        assert!(task._removed_commit.as_ref().unwrap().1.is_empty());
    } else {
        assert_eq!(index.by_commit[&commit].len(), expected_remaining);
        assert!(!index.by_commit[&commit].contains(&snapshot));
    }
    assert_eq!(task.removed.as_ref().unwrap().0, snapshot);
    assert_eq!(task._removed_member, Some(snapshot));
}

pub(in crate::relational_source) fn assert_observation_index_refusals(
    task: &mut RelationalBridgeObservationRetirement,
    source: &RuntimeBridgeRelationalSource,
    request: &ExecutionRequest<'_, '_>,
) {
    let snapshot = task.lease.snapshot_id;
    let mut index = source.observation_bindings.entries.lock().unwrap();
    let binding = index.by_snapshot.remove(&snapshot).unwrap();
    drop(index);
    assert_eq!(
        task.advance(request, source),
        Progress::Refused(Denial::RegistrationMismatch)
    );
    let mut index = source.observation_bindings.entries.lock().unwrap();
    let commit = binding.commit_id.unwrap();
    index.by_snapshot.insert(snapshot, binding);
    let member = index
        .by_commit
        .get_mut(&commit)
        .unwrap()
        .take(&snapshot)
        .unwrap();
    drop(index);
    assert_eq!(
        task.advance(request, source),
        Progress::Refused(Denial::CommitIndexMismatch)
    );
    let mut index = source.observation_bindings.entries.lock().unwrap();
    assert!(index.by_snapshot.contains_key(&snapshot));
    index.by_commit.get_mut(&commit).unwrap().insert(member);
}
