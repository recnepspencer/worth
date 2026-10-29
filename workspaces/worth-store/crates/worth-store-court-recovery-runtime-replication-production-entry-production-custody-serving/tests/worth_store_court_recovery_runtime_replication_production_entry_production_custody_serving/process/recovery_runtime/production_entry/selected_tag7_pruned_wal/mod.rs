//! A native cutover may retain tag-7 custody after its old C.9 WAL is gone.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::selected_tag7_pruned_wal::*;
use {std::fs, std::num::NonZeroU64, std::path::Path};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

#[test]
fn native_cutover_prunes_old_released_wal_but_selected_tag7_reopens() {
    // The native policy is large enough for every real V3 frame, but bounded
    // so a later checkpoint can retire its fully covered old segment.
    let world =
        super::pending_wal_world::first_with_wal_segment_bytes(std::num::NonZeroU64::new(1 << 20).unwrap());
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("first killed V3 must recover before native cutover: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .unwrap();
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal_and_wal_segment_bytes(
        world.root(),
        seal,
        std::num::NonZeroU64::new(1 << 20).unwrap(),
    );
    serving
        .retire_displaced_segment()
        .expect("native retirement must publish the first Batch checkpoint");
    let c1 = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (first_batches, first_accumulator) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&c1);
    assert_eq!(first_batches.len(), 1);
    assert_eq!(
        first_accumulator.base().tip(),
        first_batches[0].tip_provenance().unwrap()
    );
    let first_heads = worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
        world.root(),
        first_accumulator,
    );

    // Only a sealed prefix can be reclaimed. Complete enough ordinary
    // publications to rotate the 1 MiB WAL segment containing the old V3;
    // unresolved durability-only padding would keep that prefix live.
    rotate_completed_wal(&serving);

    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x9d; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("idle accumulator-only checkpoint must admit after native cutover");
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();
    let c2 = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    assert_ne!(c2, c1);
    let (idle_batches, idle_accumulator) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&c2);
    assert!(idle_batches.is_empty());
    assert_eq!(
        idle_accumulator.base().tip(),
        first_accumulator.base().tip()
    );
    assert_eq!(
        idle_accumulator.prior_head_count(),
        first_accumulator.head_count()
    );
    assert_eq!(
        idle_accumulator.prior_head_roster_digest(),
        first_accumulator.head_roster_digest()
    );
    let heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(world.root(), idle_accumulator);
    assert_eq!(heads, first_heads);
    let witness = idle_accumulator.base().tip().fate();
    assert!(
        !retained_wal_interval(
            world.root(),
            witness.lsn_start(),
            witness.lsn_end_exclusive(),
        ),
        "native cutover must physically prune the old V3 WAL frame",
    );

    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("selected tag-7 without old WAL must reopen: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .unwrap();
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal_and_wal_segment_bytes(
        world.root(),
        seal,
        std::num::NonZeroU64::new(1 << 20).unwrap(),
    )
    .close();
}
