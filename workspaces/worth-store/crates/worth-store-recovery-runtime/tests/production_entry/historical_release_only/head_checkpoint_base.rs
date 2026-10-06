//! Completed B history must rejoin independently from A's selected head checkpoint.

use super::*;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest,
};

#[test]
fn released_checkpoint_base_survives_completed_postcheckpoint_release() {
    std::thread::Builder::new()
        .name("completed-head-checkpoint-base".into())
        .stack_size(16 << 20)
        .spawn(run)
        .unwrap()
        .join()
        .expect("completed HeadV2 recovery worker");
}

fn run() {
    let world = pending_wal_world::first();
    let root = world.root();

    // A is a real pending V3. C8 performs its root transition; Store seals
    // Serving, which then publishes the first head-bearing checkpoint.
    let first = recover(root, "A pending WAL");
    assert!(first.core().recovery_effect_count() > 0);
    let seal = first.into_core().into_checkpoint_custody().unwrap();
    let serving = certified_release_serving::admit_serving_with_seal(root, seal);
    checkpoint(&serving, [0xca; 32]);
    serving.close();

    let checkpoint_bytes = fs::read(root.join("families/checkpoint.current")).unwrap();
    let (a_batches, a_accumulator) =
        release_reopen::selected_release_certificates_from_bytes(&checkpoint_bytes);
    assert_eq!(a_batches.len(), 1, "A must be checkpoint-attested");
    let a_heads = release_reopen::selected_head_oracle::selected_heads(root, a_accumulator);
    let [a_head] = a_heads.as_slice() else {
        panic!("the checkpoint must carry exactly A's head")
    };
    assert!(!a_head.terminal());

    // A separate process opens that HeadV2 checkpoint, genuinely ingests B,
    // and is killed at B's durable descriptor WAL before another checkpoint.
    world.kill_distinct_release_before_checkpoint();
    assert_eq!(
        fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint_bytes,
        "B's interrupted release must leave A as the selected checkpoint source"
    );

    let second = recover(root, "B pending WAL");
    assert!(second.core().recovery_effect_count() > 0);
    let seal = second.into_core().into_checkpoint_custody().unwrap();
    certified_release_serving::admit_serving_with_seal(root, seal).close();
    let selected = selected_generation(root);
    let expected_heads = completed_tip::selected_root_heads(root);
    assert_eq!(expected_heads.len(), 2);
    assert_eq!(
        expected_heads
            .iter()
            .find(|head| head.key() == a_head.key()),
        Some(a_head),
        "B's release must carry A's exact checkpoint head"
    );
    let b_head = expected_heads
        .iter()
        .find(|head| head.key() != a_head.key())
        .expect("B's distinct head");
    assert_eq!(b_head.cumulative_dropped(), 1);
    assert_eq!(
        fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint_bytes,
        "completed B history must still have A's HeadV2 checkpoint source"
    );

    // This fresh C8 path now has completed B history, not a pending descriptor.
    // Store must independently rejoin A's selected head and B's transition.
    let completed = recover(root, "completed B history from HeadV2");
    assert_eq!(completed.core().recovery_effect_count(), 0);
    assert_eq!(selected_generation(root), selected);
    assert_eq!(
        fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint_bytes
    );
    let seal = completed.into_core().into_checkpoint_custody().unwrap();
    let serving = certified_release_serving::admit_serving_with_seal(root, seal);
    checkpoint(&serving, [0xcb; 32]);
    serving.close();

    let (_, accumulator) = release_reopen::selected_release_certificates_from_bytes(
        &fs::read(root.join("families/checkpoint.current")).unwrap(),
    );
    assert_eq!(
        release_reopen::selected_head_oracle::selected_heads(root, accumulator),
        expected_heads,
        "successor checkpoint must preserve both exact per-object heads"
    );
    let final_reopen = recover(root, "head-bearing successor checkpoint");
    assert_eq!(final_reopen.core().recovery_effect_count(), 0);
    let seal = final_reopen.into_core().into_checkpoint_custody().unwrap();
    certified_release_serving::open_serving_with_seal_without_checkpoint(root, seal);
}

fn recover(
    root: &Path,
    stage: &str,
) -> worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff {
    match WorthStoreRecovery::recover(certified_release_serving::request(root)) {
        PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        PhysicalRecoveryOutcome::Blocked(block) => panic!(
            "{stage} blocked: kind={:?}; artifact={:?}; cause={:?}; effects={}",
            block.cause(),
            block.evidence().artifact.as_deref(),
            block.evidence().planning_denial,
            block.recovery_effects()
        ),
        PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
            "{stage} indeterminate: handoff={:?}; reopen={:?}; effects={}",
            failure.handoff_failure(),
            failure.reopen_failure(),
            failure.recovery_effects()
        ),
        _ => panic!("{stage} did not recover"),
    }
}

fn checkpoint(serving: &worth_store::physical_runtime::ServingPhysicalRuntime, key: [u8; 32]) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("head-bearing checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}
