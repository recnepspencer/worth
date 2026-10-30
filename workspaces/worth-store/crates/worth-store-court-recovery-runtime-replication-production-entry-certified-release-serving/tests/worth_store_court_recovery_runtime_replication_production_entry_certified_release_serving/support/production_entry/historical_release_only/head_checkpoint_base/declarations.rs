//! Completed B history must rejoin independently from A's selected head checkpoint.

pub(crate) use {
    crate::production_entry::historical_release_only::completed_tip,
    crate::production_entry::historical_release_only::selected_generation, std::fs,
    std::path::Path, worth_proof::TransitionOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::pending_wal_world,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen,
};
pub(crate) use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};

pub(crate) fn run() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first();
    let root = world.root();

    // A is a real pending V3. C8 performs its root transition; Store seals
    // Serving, which then publishes the first head-bearing checkpoint.
    let first = recover(root, "A pending WAL");
    assert!(first.core().recovery_effect_count() > 0);
    let seal = first
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .unwrap();
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal);
    checkpoint(&serving, [0xca; 32]);
    serving.close();

    let checkpoint_bytes = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    let (a_batches, a_accumulator) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&checkpoint_bytes);
    assert_eq!(a_batches.len(), 1, "A must be checkpoint-attested");
    let a_heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            root,
            a_accumulator,
        );
    let [a_head] = a_heads.as_slice() else {
        panic!("the checkpoint must carry exactly A's head")
    };
    assert!(!a_head.terminal());

    // A separate process opens that HeadV2 checkpoint, genuinely ingests B,
    // and is killed at B's durable descriptor WAL before another checkpoint.
    world.kill_distinct_release_before_checkpoint();
    assert_eq!(
        std::fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint_bytes,
        "B's interrupted release must leave A as the selected checkpoint source"
    );

    let second = recover(root, "B pending WAL");
    assert!(second.core().recovery_effect_count() > 0);
    let seal = second
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .unwrap();
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal).close();
    let selected = selected_generation(root);
    let expected_heads =
        crate::production_entry::historical_release_only::completed_tip::selected_root_heads(root);
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
        std::fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint_bytes,
        "completed B history must still have A's HeadV2 checkpoint source"
    );

    // This fresh C8 path now has completed B history, not a pending descriptor.
    // Store must independently rejoin A's selected head and B's transition.
    let completed = recover(root, "completed B history from HeadV2");
    assert_eq!(completed.core().recovery_effect_count(), 0);
    assert_eq!(selected_generation(root), selected);
    assert_eq!(
        std::fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint_bytes
    );
    let seal = completed
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .unwrap();
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal);
    checkpoint(&serving, [0xcb; 32]);
    serving.close();

    let (_, accumulator) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(
        &std::fs::read(root.join("families/checkpoint.current")).unwrap(),
    );
    assert_eq!(
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            root,
            accumulator
        ),
        expected_heads,
        "successor checkpoint must preserve both exact per-object heads"
    );
    let final_reopen = recover(root, "head-bearing successor checkpoint");
    assert_eq!(final_reopen.core().recovery_effect_count(), 0);
    let seal = final_reopen
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .unwrap();
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(root, seal);
}

pub(crate) fn recover(
    root: &Path,
    stage: &str,
) -> worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff {
    match worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root)) {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => *handoff,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
            "{stage} blocked: kind={:?}; artifact={:?}; cause={:?}; effects={}",
            block.cause(),
            block.evidence().artifact.as_deref(),
            block.evidence().planning_denial,
            block.recovery_effects()
        ),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
            "{stage} indeterminate: handoff={:?}; reopen={:?}; effects={}",
            failure.handoff_failure(),
            failure.reopen_failure(),
            failure.recovery_effects()
        ),
        _ => panic!("{stage} did not recover"),
    }
}

pub(crate) fn checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    key: [u8; 32],
) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("head-bearing checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}
