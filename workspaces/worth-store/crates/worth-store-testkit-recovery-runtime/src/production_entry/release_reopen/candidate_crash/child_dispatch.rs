//! Shared subprocess roles, mounted by each consuming court.
use super::*;

pub fn dispatch_candidate_certificate_serving_child() {
    let Some(root) = std::env::var_os(RECOVERY_CHILD_ROOT) else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    std::thread::Builder::new()
        .name("candidate-crash-recovery-serving".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
                crate::production_entry::certified_release_serving::recovery_request::request(&root),
            );
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome
            else {
                panic!("selected predecessor recovery denied: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .certification_release_media_into_checkpoint_custody()
                .expect("selected predecessor Store custody seal");
            if std::env::var_os(RECOVERY_CHILD_CHECKPOINT).is_some() {
                crate::production_entry::certified_release_serving::open_serving_with_seal(&root, seal);
            } else {
                crate::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(
                    &root, seal,
                );
            }
            eprintln!("C8_CANDIDATE_CRASH_SELECTED_PREDECESSOR_OPENED");
        })
        .expect("spawn bounded recovery stack")
        .join()
        .expect("recovery Serving worker");
}

pub fn dispatch_candidate_certificate_child() {
    let Some(marker) = std::env::var_os(CHILD_MARKER) else {
        return;
    };
    let marker = std::path::PathBuf::from(marker);
    let (world, receipt, _) = released_world(1024);
    assert_eq!(
        receipt.remaining_payload_records(),
        0,
        "candidate crash needs a terminal release"
    );
    let (_, accumulator) = selected_release_certificates(&world);
    assert!(accumulator.base().terminal());
    let gate = world.serving().pause_physical_checkpoint_at(
        worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidateCertificate,
    );
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x75; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("release-custody successor checkpoint must admit")
    };
    assert!(
        gate.await_arrival(),
        "candidate tag-7 append was not reached"
    );
    std::fs::write(
        marker.join("root.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .expect("write exact child root path");
    std::fs::write(marker.join("candidate-certificate.reached"), b"reached")
        .expect("write reached marker after production pause");
    let _handle = handle;
    loop {
        std::thread::park();
    }
}
