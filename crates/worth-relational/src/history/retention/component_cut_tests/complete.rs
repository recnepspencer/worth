//! Coordinate a Complete capture with the actual external retention cut.
use super::super::capture_pause::CapturePause;
use crate::branch::basis::RelationalBranchBasisDescriptor;
use std::sync::{mpsc::sync_channel, Arc, Mutex};

/// Exact private assertion for the controlled component SDK leaf. No owner or
/// request is constructed here, and the callback returns descriptive DATA only.
pub(crate) fn assert_controlled_component_capture_winner(
    history: &crate::runtime::state::subsystems::history::HistorySubsystem,
    instrumentation: &crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    basis_b: &crate::branch::basis::AdmittedRelationalBranchBasis,
    lease_a: crate::history::retention::RelationalBranchRetentionLease,
    capture: impl FnOnce() -> Vec<RelationalBranchBasisDescriptor> + Send,
) {
    let owner = history.retention_binding().owner.upgrade().unwrap();
    let coordinate = |descriptor: &RelationalBranchBasisDescriptor| {
        (
            descriptor.branch_id().clone(),
            descriptor.reference().generation(),
        )
    };
    let first = coordinate(lease_a.descriptor());
    let second = coordinate(basis_b.descriptor());
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
            crate::branch::basis_retention::release_component_basis(
                history,
                history.runtime_instance_id,
                lease_a,
            )
            .unwrap();
            crate::branch::basis_retention::retain_component_basis(
                history,
                history.runtime_instance_id,
                instrumentation,
                basis_b,
            )
            .unwrap()
        });
        started_rx.recv().unwrap();
        resume_tx.send(()).unwrap();
        let selected = selecting.join().unwrap();
        let selected = selected.iter().map(coordinate).collect::<Vec<_>>();
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
        crate::branch::basis_retention::release_component_basis(
            history,
            history.runtime_instance_id,
            lease_b,
        )
        .unwrap();
    });
}
