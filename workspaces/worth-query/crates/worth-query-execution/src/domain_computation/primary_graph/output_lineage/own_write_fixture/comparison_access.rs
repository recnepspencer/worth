//! Stale performed postconditions cannot enter comparison or dirty clearance.
use super::*;

#[test]
fn own_write_cutoff_cannot_project_retained_facts_for_comparison() {
    with_generated_own_write(|world, _, cell, _| {
        let handle = world
            .application
            .runtime
            .primary_graph()
            .unwrap()
            .integration_handle();
        let owner = &handle.source_owner.invalidation_owner;
        let candidate =
            super::super::input_cutoff::RetainedInputCutoffCandidate::from_exact_cell(cell);
        assert!(
            candidate
                .observed_source_facts(&mut owner.edit_admission())
                .unwrap()
                .is_none(),
            "postconditions of a stale computation are not comparable input facts"
        );
    });
}

#[test]
fn own_write_dirty_marks_cannot_be_cleared_by_postcondition_comparison() {
    with_committed_own_write(|world, _, cell, _| {
        let handle = world
            .application
            .runtime
            .primary_graph()
            .unwrap()
            .integration_handle();
        let owner = &handle.source_owner.invalidation_owner;
        handle.with_runtime(|runtime| {
            let basis = runtime
                .admit_branch_basis(&runtime.main_branch_identity())
                .unwrap();
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap();
            let selected = runtime.read_truth().positioned_snapshot(&snapshot).unwrap();
            let answer = owner.reverify_dirty(
                runtime,
                &snapshot,
                &selected,
                &cell.get().unwrap().settlement_identity,
                &mut owner.edit_admission(),
            );
            assert!(
                matches!(
                    answer,
                    Ok(super::super::invalidation::DirtyReverification::ChangedComputation)
                ),
                "comparison of rebased x=2 cannot discharge a computation made from x=1"
            );
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        });
    });
}
