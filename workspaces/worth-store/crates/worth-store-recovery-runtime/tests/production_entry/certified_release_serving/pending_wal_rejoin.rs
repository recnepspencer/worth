//! A post-claim selected-media drift must not cross Store's independent
//! pending-WAL rejoin or create a checkpoint-capable Serving instance.

use super::*;

#[test]
fn pending_v3_sealed_serving_can_checkpoint_the_recovered_release() {
    let world = super::super::pending_wal_world::first();
    let root = world.root().to_path_buf();
    let worker = std::thread::Builder::new()
        .name("pending-v3-sealed-serving-checkpoint".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let checkpoint_path = root.join("families/checkpoint.current");
            let before = std::fs::read(&checkpoint_path).expect("selected NoRelease checkpoint");
            let outcome = WorthStoreRecovery::recover(request(&root));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("real pending V3 C8 recovery failed: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .into_checkpoint_custody()
                .expect("pending V3 selected custody seal");
            open_serving_with_seal(&root, seal);
            let after = std::fs::read(&checkpoint_path).expect("selected release checkpoint");
            assert_ne!(after, before, "Serving must publish a new checkpoint");
            let (batches, accumulator) =
                super::super::release_reopen::selected_release_certificates_from_bytes(&after);
            assert_eq!(batches.len(), 1, "pending V3 must become one real Batch");
            let batch = batches[0];
            assert_eq!(batch.ordinal(), 0);
            assert_eq!(batch.predecessor(), None);
            assert_eq!(accumulator.base().batch_count(), 1);
            assert_eq!(accumulator.base().tip(), batch.tip_provenance().unwrap());
            assert_eq!(
                accumulator.base().cumulative_dropped(),
                batch.cumulative_dropped()
            );
            assert_eq!(
                accumulator.base().cumulative_digest(),
                batch.cumulative_digest()
            );
            let heads = super::super::release_reopen::selected_head_oracle::selected_heads(
                &root,
                accumulator,
            );
            assert_eq!(heads.len(), 1);
        })
        .expect("pending sealed-serving worker");
    worker
        .join()
        .expect("pending sealed-serving worker did not panic");
}

#[test]
fn pending_v3_reservation_drift_after_c8_claim_denies_store_seal_and_plain_checkpoint() {
    let world = super::super::pending_wal_world::first();
    let root = world.root().to_path_buf();
    let worker = std::thread::Builder::new()
        .name("pending-v3-selected-media-drift".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let after_claim = Arc::new(AtomicBool::new(false));
            let before_final = Arc::new(AtomicBool::new(false));
            let reservation = Arc::new(Mutex::new(None));
            let claim_flag = Arc::clone(&after_claim);
            let claim_reservation = Arc::clone(&reservation);
            let final_flag = Arc::clone(&before_final);
            let final_reservation = Arc::clone(&reservation);
            let final_root = root.clone();
            let outcome = WorthStoreRecovery::certification_recover_with_custody_pauses(
                request(&root),
                move |route| {
                    claim_flag.store(true, Ordering::SeqCst);
                    *claim_reservation.lock().unwrap() = Some(route);
                },
                move || {
                    final_flag.store(true, Ordering::SeqCst);
                    tamper::substitute_selected_reservation(
                        &final_root,
                        final_reservation
                            .lock()
                            .unwrap()
                            .expect("C8 selected reservation"),
                    );
                },
            );
            assert!(
                after_claim.load(Ordering::SeqCst),
                "C8 pending claim not reached"
            );
            assert!(
                before_final.load(Ordering::SeqCst),
                "Store final rejoin not reached"
            );
            let PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) = outcome else {
                panic!("changed selected reservation must deny before C8 seal: {outcome:?}")
            };
            assert_eq!(
                indeterminate.handoff_failure(),
                Some(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch),
            );
            assert_plain_serving_checkpoint_unavailable(&root);
        })
        .expect("pending drift recovery worker");
    worker
        .join()
        .expect("pending drift recovery worker did not panic");
}
