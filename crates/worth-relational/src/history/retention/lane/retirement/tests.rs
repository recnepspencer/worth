//! The real lane lock cannot turn a cooperative release into a wait.
use super::*;
use crate::branch::basis_admission_tests::observed_empty_branch;

#[test]
fn result_snapshot_cleanup_lane_contention_keeps_original_obligation() {
    let (_history, basis) = observed_empty_branch(1);
    let binding = &basis.inner.retention_binding;
    let mut obligation = binding
        .acquire(
            RelationalRetentionObligationKind::Observation,
            vec![basis.inner.root.clone()],
            None,
        )
        .unwrap();
    let lane = obligation.lane.clone();
    let locked = lane.state.lock().unwrap();
    let before = locked.obligations.len();
    assert!(!obligation.try_retire_observation());
    assert_eq!(locked.obligations.len(), before);
    assert!(locked.obligations.contains_key(&obligation.obligation_id));
    assert!(!obligation.terminal);
    assert_eq!(obligation.roots.len(), 1);
    drop(locked);
    assert!(obligation.try_retire_observation());
    assert!(obligation.terminal);
    assert!(obligation.roots.is_empty());
    assert!(!lane
        .state
        .lock()
        .unwrap()
        .obligations
        .contains_key(&obligation.obligation_id));
    assert!(obligation.try_retire_observation());
    drop(basis);
}
