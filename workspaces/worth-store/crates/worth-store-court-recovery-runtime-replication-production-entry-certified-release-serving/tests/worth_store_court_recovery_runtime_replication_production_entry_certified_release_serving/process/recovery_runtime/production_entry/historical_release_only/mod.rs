//! A completed release history followed by an ordinary selected root has no
//! pending V3 descriptor; it still owes an independently rejoined Store seal.

#[allow(unused_imports)] use crate::production_entry::historical_release_only::*;
use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world};
use {std::fs, std::path::Path};
use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch};
use {worth_store_physical_format::DurableRootSelector, worth_store_physical_format::RecordArtifactFile};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};





#[test]
fn two_released_batches_then_ordinary_append_reopen_with_historical_seal() {
    let world = pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("two genuine released batches must recover: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("two-batch release must produce a Store seal");
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let checkpoint_before = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let selected_before = selected_generation(world.root());

    let format = worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(format))
        .unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xc8; 32]))
        .unwrap();
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"ordinary-after-two-v3".as_slice()]).unwrap(),
                placement,
                worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                    key,
                    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("ordinary append after two V3 releases must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
    assert_eq!(
        std::fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint_before,
        "ordinary append must not silently create a release checkpoint",
    );
    assert!(
        selected_generation(world.root()) > selected_before,
        "ordinary append must advance the selected root",
    );
    serving.close();
    let completed_selected = selected_generation(world.root());

    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        if let worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(failure) = &outcome {
            panic!("historical-only completed releases must recover: handoff={:?}; reopen={:?}; effects={}",
                failure.handoff_failure(), failure.reopen_failure(), failure.recovery_effects());
        }
        panic!("historical-only completed releases must recover: {outcome:?}");
    };
    assert_eq!(handoff.core().recovery_effect_count(), 0);
    assert_eq!(selected_generation(world.root()), completed_selected);
    assert_eq!(
        std::fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint_before,
    );
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("ordinary selected tip must preserve the historical release seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}
#[path = "completed_freshness/mod.rs"]
pub(crate) mod completed_freshness;
#[path = "completed_tip/mod.rs"]
pub(crate) mod completed_tip;
#[path = "head_checkpoint_base/mod.rs"]
pub(crate) mod head_checkpoint_base;
