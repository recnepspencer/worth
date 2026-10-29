//! Actual checkpoint-source rereads retain their responsible failure boundary.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::head_diagnostics::*;
use {std::fs, std::path::Path, std::path::PathBuf};

use {worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalCheckpointSource, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES};
use {worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial, worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadControlDenial as Control, worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadWalkDenial as Walk, worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial as Denial, worth_store_recovery_runtime::WorthStoreRecovery};

use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, worth_store_testkit_recovery_runtime::certified_release_serving::tamper};

#[test]
fn source_root_head_and_control_failures_remain_distinct_and_deny_serving() {
    let (world, first, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
    assert!(first.remaining_payload_records() > 0);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xea; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("actual partial-drop checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    std::thread::Builder::new()
        .name("selected-head-diagnostics".to_owned())
        .stack_size(16 << 20)
        .spawn(move || hostile_rereads(&root))
        .unwrap()
        .join()
        .unwrap();
}
