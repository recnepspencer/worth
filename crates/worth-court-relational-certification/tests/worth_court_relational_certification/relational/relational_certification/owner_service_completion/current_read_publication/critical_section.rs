//! Observe the real readonly source lock at the receiving owner's cutover.
use super::*;
use std::sync::{mpsc::sync_channel, Arc};
use std::time::Duration;
use worth_relational::facade::mvcc::{
    RelationalOperationControl, RelationalPatchPositionReservationGateForTest,
};

struct ReleaseOnDrop(Arc<RelationalPatchPositionReservationGateForTest>);
impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.0.open();
    }
}

#[test]
fn read_and_target_cells_stay_locked_at_actual_cutover_in_both_orders_and_same_cell() {
    for branch in ["aaa", "zzz", "main"] {
        let host = Arc::new(serial_request());
        let execution = &ExecutionRequest::serial(&host);
        let runtime = fixture().build_runtime();
        let original = create(execution, &runtime, "control", "allowed");
        let source = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let identity = if branch == "main" {
            runtime.main_branch_identity()
        } else {
            fork(execution, &runtime, branch)
        };
        let target = runtime.admit_branch_basis(&identity).unwrap();
        assert!(!source.publication_state_locked_for_test());
        assert!(!target.publication_state_locked_for_test());
        let gate = Arc::new(RelationalPatchPositionReservationGateForTest::default());
        let release = ReleaseOnDrop(Arc::clone(&gate));
        let (reached_tx, reached_rx) = sync_channel(1);
        let control = RelationalOperationControl::uninterrupted()
            .with_patch_position_reservation_pause_for_test(reached_tx, Arc::clone(&gate));
        let mut candidate = prepare_with_control(execution, &runtime, &target, control);
        candidate.require_current_read(&source).unwrap();
        let port = runtime.publication_port();
        let worker = std::thread::spawn({
            let host = Arc::clone(&host);
            move || port.compare_and_publish(&ExecutionRequest::serial(&host), candidate)
        });
        let reached = reached_rx.recv_timeout(Duration::from_secs(10));
        // The sampled fact is the actual state mutex, not the unrelated global
        // patch-position reservation that would also defer another publisher.
        let source_locked = reached.is_ok() && source.publication_state_locked_for_test();
        let target_locked = reached.is_ok() && target.publication_state_locked_for_test();
        drop(release);
        let outcome = worker
            .join()
            .expect("the real publication worker completes");
        assert!(
            reached.is_ok(),
            "the original receiver never reached its actual cutover"
        );
        assert!(
            source_locked,
            "C was not protected before the D replacement"
        );
        assert!(target_locked, "D was not protected at its own replacement");
        assert!(!source.publication_state_locked_for_test());
        assert!(!target.publication_state_locked_for_test());
        let RelationalPublicationOutcome::Performed(performed) = outcome else {
            panic!("the lawful actual cutover must perform: {outcome:?}");
        };
        let result = runtime
            .settlement_port()
            .settle_performed_publication(performed)
            .unwrap();
        let output = changed_entities(&result)[0];
        assert_eq!(
            values(execution, &runtime, &result, [output, output]),
            ["produced", "produced"].map(string_aspect_value)
        );
        release_test_commit_snapshot(&runtime, &result);
        release_test_commit_snapshot(&runtime, &original);
    }
}
