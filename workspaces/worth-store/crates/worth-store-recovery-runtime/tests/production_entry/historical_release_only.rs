//! A completed release history followed by an ordinary selected root has no
//! pending V3 descriptor; it still owes an independently rejoined Store seal.

use super::*;
use std::{fs, path::Path};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalRecordFormatDeclaration, PhysicalRecordPlacementPolicy, RecordAppendBatch,
};
use worth_store_physical_format::{DurableRootSelector, RecordArtifactFile};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[test]
fn two_released_batches_then_ordinary_append_reopen_with_historical_seal() {
    let world = pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("two genuine released batches must recover: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("two-batch release must produce a Store seal");
    let serving = certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let checkpoint_before = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let selected_before = selected_generation(world.root());

    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(format))
        .unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xc8; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"ordinary-after-two-v3".as_slice()]).unwrap(),
                placement,
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("ordinary append after two V3 releases must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
    assert_eq!(
        fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint_before,
        "ordinary append must not silently create a release checkpoint",
    );
    assert!(
        selected_generation(world.root()) > selected_before,
        "ordinary append must advance the selected root",
    );
    serving.close();

    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("historical-only completed releases must recover: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("ordinary selected tip must preserve the historical release seal");
    certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

fn selected_generation(root: &Path) -> u64 {
    let bytes = fs::read(
        root.join("families/records")
            .join(RecordArtifactFile::CurrentRootSelector.file_name()),
    )
    .unwrap();
    DurableRootSelector::decode(&bytes)
        .unwrap()
        .root_generation()
}
