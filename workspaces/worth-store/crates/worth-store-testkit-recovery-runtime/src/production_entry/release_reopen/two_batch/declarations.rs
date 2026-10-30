//! Two genuine Store releases retain an older selected V3 descriptor when
//! the accumulator tip advances, including after Batch carryforward.

pub use {
    crate::production_entry::certified_release_serving,
    crate::production_entry::release_reopen::reissue_proof,
    crate::production_entry::release_reopen::released_world_in_tier,
    crate::production_entry::release_reopen::selected_release_certificates,
    crate::release_reopen::selected_head_oracle, std::num::NonZeroU16, std::num::NonZeroU64,
    worth_proof::TransitionOutcome, worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};

pub fn run(carryforward: bool) {
    let (world, first, _, proof) = released_world_in_tier(1, false, None);
    assert!(first.remaining_payload_records() > 0);
    let (first_batches, first_accumulator) = selected_release_certificates(&world);
    assert_eq!(first_batches.len(), 1);
    let first_heads = crate::release_reopen::selected_head_oracle::selected_heads(
        world.retained_root().path(),
        first_accumulator,
    );
    let continuation = worth_store::physical_runtime::BlobReclaimRequest::released(
        reissue_proof(&proof),
        world.placement(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(1024).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
            std::num::NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    let second = world
        .serving()
        .blobs()
        .unwrap()
        .reclaim(continuation)
        .expect("same released publication continuation")
        .wait()
        .expect("second release completed");
    assert_eq!(
        second.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert_eq!(second.dropped_records().len(), 1);
    assert!(second.remaining_payload_records() < first.remaining_payload_records());
    let (batches, accumulator) = selected_release_certificates(&world);
    assert_eq!(batches.len(), 1, "second release has one current Batch");
    assert_eq!(
        accumulator.base().tip(),
        batches[0].tip_provenance().unwrap()
    );
    assert_ne!(accumulator.base().tip(), first_accumulator.base().tip());
    assert!(accumulator.base().prior_checkpoint_sequence() > 0);
    assert_eq!(
        accumulator.base().prior_cumulative_dropped(),
        first_accumulator.base().cumulative_dropped()
    );
    assert!(
        accumulator.base().cumulative_dropped() > first_accumulator.base().cumulative_dropped()
    );
    let current_heads = crate::release_reopen::selected_head_oracle::selected_heads(
        world.retained_root().path(),
        accumulator,
    );
    assert_eq!(first_heads.len(), 1);
    assert_eq!(current_heads.len(), 1);
    assert_eq!(current_heads[0].key(), first_heads[0].key());
    assert!(current_heads[0].cumulative_dropped() > first_heads[0].cumulative_dropped());
    if carryforward {
        let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
            worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x74; 32]),
            worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
                .unwrap(),
        );
        let worth_proof::TransitionOutcome::Success(handle) =
            world.serving().checkpoints().start(checkpoint).into_raw()
        else {
            panic!("carryforward checkpoint must admit")
        };
        assert!(matches!(
            handle.wait(),
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ));
        let (carried_batches, carried) = selected_release_certificates(&world);
        assert!(carried_batches.is_empty());
        assert_eq!(carried.base().tip(), accumulator.base().tip());
        assert!(carried.base().prior_checkpoint_sequence() > 0);
        assert_eq!(carried.prior_head_count(), accumulator.head_count());
        assert_eq!(
            carried.prior_head_roster_digest(),
            accumulator.head_roster_digest()
        );
    }
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    recover(root);
}

#[expect(
    clippy::disallowed_methods,
    reason = "Provides the fixture stack for production recovery of the two-batch world."
)]
pub fn recover(root: std::path::PathBuf) {
    let worker = std::thread::Builder::new()
        .name("two-release-c8-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::
                certification_recover_with_custody_pauses(
                    crate::production_entry::certified_release_serving::recovery_request::request(&root),
                    |_| {},
                    || {},
                );
            match outcome {
                worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(_) => {}
                worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                    indeterminate,
                ) => panic!(
                    "two-release handoff failure: {:?}; reopen failure: {:?}",
                    indeterminate.handoff_failure(),
                    indeterminate.reopen_failure()
                ),
                other => panic!("genuine two-release C8 recovery: {other:?}"),
            }
        })
        .expect("recovery worker");
    worker.join().expect("recovery worker");
}
