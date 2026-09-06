use super::*;
use std::sync::Arc;
use worth_relational::facade::mvcc::{
    RelationalOperationControl, RelationalPatchPositionReservationGate,
};

struct ReleaseRelational(Arc<RelationalPatchPositionReservationGate>);
impl Drop for ReleaseRelational {
    fn drop(&mut self) {
        self.0.open();
    }
}

#[test]
fn combined_loser_retains_relational_movement_without_calling_signal_after_head_change() {
    let mut court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let (reached, received) = mpsc::sync_channel(1);
    let gate = Arc::new(RelationalPatchPositionReservationGate::default());
    let candidate = court.records.candidate_with_control(
        root.basis().relational_basis(),
        "grain",
        "5",
        RelationalOperationControl::uninterrupted()
            .with_patch_position_reservation_pause(reached, Arc::clone(&gate)),
    );
    let combined = court
        .world
        .publication_port()
        .prepare_with_signal(
            root.clone(),
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(candidate),
            &token,
            None,
        )
        .unwrap();
    let single = signal_prepared(&court, &root, &token);
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .canonical_movements();
    let mut context = court.context(&root);
    let port = court.world.publication_port();
    std::thread::scope(|scope| {
        let release = ReleaseRelational(gate);
        let worker = scope.spawn(move || {
            port.execute_with_signal(combined, &mut context, &token, |_| {
                panic!("stale combined intent must not enter Signal")
            })
        });
        received
            .recv_timeout(WAIT)
            .expect("real Relational reservation reached");
        assert_eq!(
            court.observe(&root).selected_commit(),
            root.selected_commit()
        );
        let RuntimeWorldPublicationOutcome::Performed(winner) =
            court.world.publication_port().execute_with_signal(
                single,
                &mut court.context(&root),
                &RuntimeWorldCancellationSource::new().token(),
                |_| Ok(()),
            )
        else {
            panic!("Signal-only contender wins");
        };
        let head = court.observe(&root);
        assert_eq!(head.selected_commit(), winner.commit().identity());
        drop(release);
        let RuntimeWorldPublicationOutcome::ProductUnpublished(loser) = worker.join().unwrap()
        else {
            panic!("combined loser retains the real Relational movement");
        };
        assert_eq!(loser.owner_effect_count(), 1);
        assert_eq!(loser.cause(), ProductUnpublishedCause::StaleProductHead);
        assert!(loser.product_comparison_costs().is_none());
        assert_eq!(
            signal
                .owner_service_cost_snapshot()
                .unwrap()
                .canonical_movements(),
            before + 1
        );
        let direct = court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .1;
        assert_eq!(
            court
                .records
                .read(&direct)
                .records
                .get("grain")
                .map(String::as_str),
            Some("5")
        );
        assert_eq!(
            court
                .records
                .read(head.basis().relational_basis())
                .records
                .get("grain")
                .map(String::as_str),
            Some("4")
        );
        clean(
            &court,
            RuntimeWorldPublicationOutcome::ProductUnpublished(loser),
        );
        assert_eq!(
            court.observe(&root).selected_commit(),
            head.selected_commit()
        );
        drop(winner.consume());
    });
    drop(root);
    court.finish();
}
