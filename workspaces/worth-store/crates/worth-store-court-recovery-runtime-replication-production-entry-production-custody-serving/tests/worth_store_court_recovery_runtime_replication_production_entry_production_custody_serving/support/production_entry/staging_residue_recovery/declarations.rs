//! A record append killed during root publication leaves root-protocol
//! candidates in record staging. Recovery removes them after its own
//! publication, so the recovered store serves instead of requiring inspection,
//! and a repeated recovery finds nothing left to remove.

pub(crate) use {std::fs, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) use {std::path::Path, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving};

pub(crate) const ROLE_ENV: &str = "WORTH_STAGING_RESIDUE_ROLE";
pub(crate) const MARKER_ENV: &str = "WORTH_STAGING_RESIDUE_MARKER";
pub(crate) const CHILD_TEST: &str = "staging_residue_recovery::staging_residue_child";

pub(crate) fn append(world: &PhysicalResidencyStoreWorld, key: u8) {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([key; 32]))
        .unwrap();
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"staging residue record"]).unwrap(),
                world.placement(),
                worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                    key,
                    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("the append must prepare")
    };
    let _ = prepared.execute();
}

pub(crate) fn serve(root: &std::path::Path, stage: &str) -> ServingPhysicalRuntime {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("{stage} did not recover: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .unwrap();
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal)
}

pub(crate) fn assert_serving_ready(serving: &ServingPhysicalRuntime) {
    assert!(serving.publication_residue().is_empty());
    assert!(serving.physical_recovery_obligations().is_empty());
    assert!(
        serving.blobs().is_ok(),
        "the recovered store requires inspection"
    );
}