//! Deterministic distinct-lane membership changes cross one real owner cut.
use crate::tests::support::*;
use std::sync::{
    mpsc::{sync_channel, Receiver, SyncSender},
    Arc, Mutex,
};

#[derive(Debug)]
pub(super) struct CapturePause {
    entered: SyncSender<()>,
    resume: Mutex<Receiver<()>>,
}
impl CapturePause {
    pub(super) fn wait(&self) {
        self.entered.send(()).unwrap();
        self.resume.lock().unwrap().recv().unwrap();
    }
}
#[test]
fn native_custody_correction_cross_lane_acquire_release_has_one_actual_capture_cut() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "lane-a-original");
    let (_, basis_a) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let lease_a = runtime.retain_component_basis(&basis_a).unwrap();
    drop(basis_a);
    let other = create_branch_from_main(&runtime, "cut-lane-b");
    let (_, basis_b) = runtime
        .observe_branch(&runtime.branch_identity(&other).unwrap())
        .unwrap();
    create_entity(execution, &runtime, "lane-a-advanced");
    let owner = runtime.history.retention_binding().owner.upgrade().unwrap();
    let (entered_tx, entered_rx) = sync_channel(0);
    let (resume_tx, resume_rx) = sync_channel(0);
    *owner.component_capture_pause.lock().unwrap() = Some(Arc::new(CapturePause {
        entered: entered_tx,
        resume: Mutex::new(resume_rx),
    }));
    let selected = std::thread::scope(|scope| {
        let capture = scope.spawn(|| runtime.history.external_checkpoint_components());
        entered_rx.recv().unwrap();
        // This receiver seam is after retired lane collection and before the
        // current inventory and all lane observations, inside the same cut.
        // It observes the real owner lock, not elapsed time or a synthetic set.
        assert!(
            owner.external_cut.try_lock().is_err(),
            "the actual capture owns the complete cut while paused between lanes"
        );
        let (started_tx, started_rx) = sync_channel(0);
        let changing_runtime = &runtime;
        let changing_basis = &basis_b;
        let changed = scope.spawn(move || {
            started_tx.send(()).unwrap();
            changing_runtime.release_component_basis(lease_a).unwrap();
            changing_runtime
                .retain_component_basis(changing_basis)
                .unwrap()
        });
        started_rx.recv().unwrap();
        resume_tx.send(()).unwrap();
        let selected = capture.join().unwrap();
        let lease_b = changed.join().unwrap();
        *owner.component_capture_pause.lock().unwrap() = None;
        assert!(selected.iter().any(
            |basis| basis.identity().branch_id() == runtime.main_branch_identity().branch_id()
        ));
        assert!(
            !selected
                .iter()
                .any(|basis| basis.identity().branch_id() == &other),
            "A and B were never simultaneous external obligations"
        );
        let after = runtime.history.external_checkpoint_components();
        assert!(!after.iter().any(
            |basis| basis.identity().branch_id() == runtime.main_branch_identity().branch_id()
        ));
        assert!(after
            .iter()
            .any(|basis| basis.identity().branch_id() == &other));
        drop(after);
        runtime.release_component_basis(lease_b).unwrap();
        selected
    });
    drop(selected);
    drop(basis_b);
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn native_custody_correction_current_lane_inventory_is_collected_inside_actual_cut() {
    let runtime = durable_runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    let retention = super::RelationalBranchRetentionOwner {
        inner: runtime.history.retention_binding().owner.upgrade().unwrap(),
    };
    let selected = retention.checkpoint_components(|| {
        assert!(
            retention.inner.external_cut.try_lock().is_err(),
            "current lane discovery must share the external retention cut"
        );
        let cell = runtime
            .history
            .branch_cell(runtime.main_branch_identity().branch_id())
            .unwrap();
        vec![(
            cell.identity().branch_id().clone(),
            cell.head_retention().binding().unwrap(),
        )]
    });
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].descriptor(), basis.descriptor());
    drop(selected);
    runtime.release_component_basis(lease).unwrap();
    drop(basis);
    runtime.prepare_close().unwrap().finish();
}

/// Exact private assertion for the controlled component SDK leaf. No owner or
/// request is constructed here, and the callback returns descriptive DATA only.
pub(crate) fn assert_controlled_component_capture_winner(
    runtime: &crate::runtime::RelationalRuntime,
    basis_b: &crate::branch::AdmittedRelationalBranchBasis,
    lease_a: super::RelationalBranchRetentionLease,
    capture: impl FnOnce() -> Vec<(
            crate::history::data::BranchId,
            crate::history::data::CommitId,
        )> + Send,
) {
    let owner = runtime.history.retention_binding().owner.upgrade().unwrap();
    let first = (
        lease_a.descriptor().branch_id().clone(),
        crate::history::data::CommitId(
            lease_a
                .descriptor()
                .reference()
                .target()
                .as_basis()
                .unwrap()
                .selected_commit_id(),
        ),
    );
    let second = (
        basis_b.descriptor().branch_id().clone(),
        crate::history::data::CommitId(
            basis_b
                .descriptor()
                .reference()
                .target()
                .as_basis()
                .unwrap()
                .selected_commit_id(),
        ),
    );
    let (entered_tx, entered_rx) = sync_channel(0);
    let (resume_tx, resume_rx) = sync_channel(0);
    *owner.component_capture_pause.lock().unwrap() = Some(Arc::new(CapturePause {
        entered: entered_tx,
        resume: Mutex::new(resume_rx),
    }));
    std::thread::scope(|scope| {
        let selecting = scope.spawn(capture);
        entered_rx.recv().unwrap();
        assert!(
            owner.external_cut.try_lock().is_err(),
            "actual controlled capture owns the complete external cut"
        );
        let (started_tx, started_rx) = sync_channel(0);
        let changing = scope.spawn(move || {
            started_tx.send(()).unwrap();
            runtime.release_component_basis(lease_a).unwrap();
            runtime.retain_component_basis(basis_b).unwrap()
        });
        started_rx.recv().unwrap();
        resume_tx.send(()).unwrap();
        let selected = selecting.join().unwrap();
        let lease_b = changing.join().unwrap();
        *owner.component_capture_pause.lock().unwrap() = None;
        assert!(
            selected.contains(&first),
            "original A external component is captured"
        );
        assert!(
            !selected.contains(&second),
            "A-old and B-old never coexist across the actual cut"
        );
        runtime.release_component_basis(lease_b).unwrap();
    });
}
