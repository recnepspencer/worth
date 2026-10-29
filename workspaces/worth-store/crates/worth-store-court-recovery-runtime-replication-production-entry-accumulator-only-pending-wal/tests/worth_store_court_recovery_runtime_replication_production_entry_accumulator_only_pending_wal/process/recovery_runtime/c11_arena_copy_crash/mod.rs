

#[allow(unused_imports)] use crate::c11_arena_copy_crash::*;
use {std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};
use {worth_store_testkit_recovery_runtime::support::EXTENT_BYTES, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store_testkit_recovery_runtime::support::append, worth_store_testkit_recovery_runtime::support::configuration, worth_store_testkit_recovery_runtime::support::generation, worth_store_testkit_recovery_runtime::support::initialize, worth_store_testkit_recovery_runtime::support::open, worth_store_testkit_recovery_runtime::support::recovery_request, worth_store_testkit_recovery_runtime::support::scan};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationRequest};
use {worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::PersistedRecordIdentity};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff, worth_store_recovery_runtime::WorthStoreRecovery};

use {crate::c11_arena_copy_crash::observation::observe_copy_media};

#[test]
fn killed_final_copy_wal_replays_one_destination_before_serving() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_after_final_wal(directory.path());
    let baseline = std::fs::read(directory.path().join("baseline-catalog")).unwrap();
    assert_eq!(
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        baseline,
        "the durable final WAL has not published a root"
    );
    observe_copy_media(&root, directory.path(), "killed");
    let record = load_identity(directory.path());
    let first = recover(&root);
    let source = selected_extent(&first, record);
    assert_eq!(
        generation(&baseline),
        first
            .selected_sources()
            .root()
            .selected()
            .selector()
            .root_generation()
    );
    drop(first);
    let second = recover(&root);
    let destination = selected_extent(&second, record);
    assert_eq!(destination.extent(), source.extent());
    assert_eq!(
        destination.extent_generation(),
        source.extent_generation() + 1
    );
    assert_ne!(
        destination.arena_range().arena(),
        source.arena_range().arena(),
        "copy evacuation publishes into another arena"
    );
    drop(second);
    let third = recover(&root);
    assert_eq!(
        selected_extent(&third, record),
        destination,
        "repeated recovery cannot publish another copy generation"
    );
    drop(third);
    let serving = open(&root);
    let rows = scan(&serving);
    assert_eq!(rows.len(), 2, "seed and selected extent each appear once");
    assert_eq!(rows.iter().filter(|(_, bytes)| bytes == b"seed").count(), 1);
    assert_eq!(
        rows.iter()
            .filter(|(_, bytes)| bytes.len() == EXTENT_BYTES && bytes.iter().all(|&byte| byte == 93))
            .count(),
        1
    );
    serving.close();
    observe_copy_media(&root, directory.path(), "recovered");
}

#[test]
#[ignore = "parent kills after final copy WAL durability and before root publication"]
fn final_copy_wal_child() {
    let directory = std::path::PathBuf::from(std::env::var_os(MARKER).unwrap());
    let root = directory.join("store");
    let serving = initialize(&root);
    let (_, placement, _) = configuration();
    append(&serving, placement, [201; 32], b"seed");
    let completed = append(&serving, placement, [202; 32], &vec![93; EXTENT_BYTES]);
    let identity = completed.persisted_records()[0];
    save_identity(&directory, identity);
    checkpoint_source(&serving);
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([203; 32]))
        .unwrap();
    let request = worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request, identity)
        .unwrap()
    else {
        panic!("selected extent copy did not admit")
    };
    for _ in 0..128 {
        if progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(progress.phase, worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption);
    let prepared = submission.prepare_completed_extent_copy().unwrap();
    let original = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    std::fs::write(directory.join("baseline-catalog"), &original).unwrap();
    let gate = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let mutation = prepared.start();
    assert!(
        gate.await_arrival(),
        "final copy did not reach durable-WAL boundary"
    );
    assert_eq!(
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        original
    );
    std::mem::forget(mutation);
    std::mem::forget(gate);
    std::fs::write(directory.join("reached"), b"final-copy-wal-durable").unwrap();
    loop {
        std::thread::park();
    }
}
#[path = "historical_displacement/mod.rs"]
pub(crate) mod historical_displacement;
#[path = "mixed_history/mod.rs"]
pub(crate) mod mixed_history;
#[path = "pre_final/mod.rs"]
pub(crate) mod pre_final;
