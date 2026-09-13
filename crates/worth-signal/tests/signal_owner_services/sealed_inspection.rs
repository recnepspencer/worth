#[cfg(feature = "test-operation-control")]
use std::panic::{catch_unwind, AssertUnwindSafe};
#[cfg(feature = "test-operation-control")]
use std::sync::mpsc;
#[cfg(feature = "test-operation-control")]
use std::thread;
#[cfg(feature = "test-operation-control")]
use std::time::Duration;

use worth_signal::facade::branch::SignalOwnerCancellationSource;
#[cfg(feature = "test-operation-control")]
use worth_signal::facade::branch::{SignalBranchRetirementReason, SignalOwnerOperationBoundary};

use super::runtime;

#[cfg(feature = "test-operation-control")]
const PROGRESS_BOUND: Duration = Duration::from_secs(3);

#[test]
fn selected_branch_resolves_live_head_after_post_seal_capture_and_restore() {
    let mut runtime = runtime();
    let root = runtime.current_branch();
    let root_basis = runtime
        .observe_signal_branch_basis(root)
        .expect("the bootstrap basis is admitted");
    let (selected, selected_basis) = runtime
        .fork_signal_branch("post-seal-selected", &root_basis)
        .expect("the selected branch is forked before sealing")
        .into_parts();
    runtime
        .switch_branch(selected.clone())
        .expect("the selected branch activates before sealing");
    let services = runtime
        .owner_component_services()
        .expect("the selected branch seals into the owner");
    let mutation = services.mutation_port();

    let (snapshot, captured_basis) = mutation
        .capture_exact(
            &selected_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("post-seal capture updates the canonical selected cell")
        .into_parts();
    let snapshot_id = snapshot.snapshot().meta.snapshot_id;
    let current_after_capture = runtime.current_branch();
    assert_eq!(
        current_after_capture,
        runtime
            .branch_handle(selected.id)
            .expect("the selected live cell remains catalogued")
    );
    assert_eq!(current_after_capture.head_snapshot_id, Some(snapshot_id));
    assert_eq!(
        runtime.branch_head_snapshot_id(selected.id),
        Some(snapshot_id)
    );

    let restored_basis = mutation
        .restore_exact(
            &captured_basis,
            &snapshot,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("post-seal restore updates the same canonical selected cell");
    assert_eq!(restored_basis.branch_id(), selected.id);
    let current_after_restore = runtime.current_branch();
    assert_eq!(
        current_after_restore,
        runtime
            .branch_handle(selected.id)
            .expect("the restored selected cell remains catalogued")
    );
    assert_eq!(current_after_restore.head_snapshot_id, Some(snapshot_id));
    assert_eq!(
        runtime.branch_head_snapshot_id(selected.id),
        Some(snapshot_id)
    );
}

#[test]
#[cfg(feature = "test-operation-control")]
fn catalog_omits_retiring_cell_and_preserves_healthy_siblings() {
    let mut runtime = runtime();
    let root = runtime.current_branch();
    let root_basis = runtime
        .observe_signal_branch_basis(root.clone())
        .expect("the bootstrap basis is admitted");
    let (retiring, retiring_basis) = runtime
        .fork_signal_branch("inspection-retiring", &root_basis)
        .expect("the retiring branch is forked")
        .into_parts();
    let (healthy, healthy_basis) = runtime
        .fork_signal_branch("inspection-healthy", &root_basis)
        .expect("the healthy sibling is forked")
        .into_parts();
    drop(root_basis);
    drop(healthy_basis);

    let services = runtime
        .owner_component_services()
        .expect("the owner seals with both siblings");
    let plan = services
        .lifecycle_port()
        .plan_retirement_exact(retiring_basis, SignalBranchRetirementReason::Superseded)
        .into_result()
        .expect("the nonselected branch has an exact retirement plan");
    let control = runtime
        .owner_operation_control()
        .expect("test operation control is issued by the sealed owner");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let lifecycle = services.lifecycle_port();
    thread::scope(|scope| {
        scope.spawn(move || {
            let result = lifecycle
                .retire_exact(plan, &SignalOwnerCancellationSource::new().token())
                .into_result();
            done_tx
                .send(result)
                .expect("retirement result is delivered");
        });
        assert!(
            pause.wait_until_reached(PROGRESS_BOUND),
            "retirement reaches its controllable cell boundary"
        );

        let known = runtime.known_branches();
        assert!(known.iter().any(|branch| branch.id == root.id));
        assert!(known.iter().any(|branch| branch.id == healthy.id));
        assert!(
            known.iter().all(|branch| branch.id != retiring.id),
            "a retiring cell is not observable in the catalog"
        );
        let ancestry = runtime.branch_ancestry(healthy.id);
        assert_eq!(
            ancestry.iter().map(|branch| branch.id).collect::<Vec<_>>(),
            vec![root.id, healthy.id],
            "an unrelated healthy lineage remains complete"
        );

        pause.release();
        assert!(done_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("retirement completes after release")
            .is_ok());
    });
    assert!(runtime.branch_handle(retiring.id).is_none());
    assert!(runtime
        .known_branches()
        .iter()
        .any(|branch| branch.id == root.id));
    assert!(runtime
        .known_branches()
        .iter()
        .any(|branch| branch.id == healthy.id));
}

#[test]
#[cfg(feature = "test-operation-control")]
fn catalog_filters_quarantined_sibling_and_ancestry_keeps_observable_suffix() {
    let mut runtime = runtime();
    let root = runtime.current_branch();
    let root_basis = runtime
        .observe_signal_branch_basis(root.clone())
        .expect("the bootstrap basis is admitted");
    let (quarantined, quarantined_basis) = runtime
        .fork_signal_branch("inspection-quarantined", &root_basis)
        .expect("the quarantined sibling is forked")
        .into_parts();
    let (grandchild, grandchild_basis) = runtime
        .fork_signal_branch("inspection-grandchild", &quarantined_basis)
        .expect("the grandchild is forked from the future quarantined parent")
        .into_parts();
    let (healthy, healthy_basis) = runtime
        .fork_signal_branch("inspection-healthy-after-quarantine", &root_basis)
        .expect("the healthy sibling is forked")
        .into_parts();
    drop(root_basis);
    drop(grandchild_basis);
    drop(healthy_basis);

    let services = runtime
        .owner_component_services()
        .expect("the owner seals with the complete lineage");
    let control = runtime
        .owner_operation_control()
        .expect("test operation control is issued by the sealed owner");
    control.inject_panic_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let mutation = services.mutation_port();
    let fault = catch_unwind(AssertUnwindSafe(|| {
        let _ = mutation.advance_exact(
            &quarantined_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        );
    }));
    assert!(
        fault.is_err(),
        "the deterministic fault quarantines one cell"
    );

    let known = runtime.known_branches();
    assert!(known.iter().any(|branch| branch.id == root.id));
    assert!(known.iter().any(|branch| branch.id == grandchild.id));
    assert!(known.iter().any(|branch| branch.id == healthy.id));
    assert!(
        known.iter().all(|branch| branch.id != quarantined.id),
        "a quarantined cell is not observable in the catalog"
    );
    let healthy_ancestry = runtime.branch_ancestry(healthy.id);
    assert_eq!(
        healthy_ancestry
            .iter()
            .map(|branch| branch.id)
            .collect::<Vec<_>>(),
        vec![root.id, healthy.id],
        "an unrelated quarantined sibling does not blank healthy ancestry"
    );
    assert_eq!(
        runtime
            .branch_ancestry(grandchild.id)
            .iter()
            .map(|branch| branch.id)
            .collect::<Vec<_>>(),
        vec![grandchild.id],
        "legacy ancestry keeps the observable suffix when its parent is quarantined"
    );
}
