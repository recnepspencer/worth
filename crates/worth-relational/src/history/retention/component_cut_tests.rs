//! Deterministic distinct-lane membership changes cross one real owner cut.
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
