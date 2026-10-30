#[allow(unused_imports)]
use crate::c11_arena_append_crash::*;
use worth_proof::TransitionOutcome;
use {
    std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread,
    std::time::Duration, std::time::Instant,
};
use {
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

use {
    worth_store_testkit_recovery_runtime::support::append,
    worth_store_testkit_recovery_runtime::support::configuration,
    worth_store_testkit_recovery_runtime::support::generation,
    worth_store_testkit_recovery_runtime::support::initialize,
    worth_store_testkit_recovery_runtime::support::open,
    worth_store_testkit_recovery_runtime::support::recovery_request,
    worth_store_testkit_recovery_runtime::support::scan,
    worth_store_testkit_recovery_runtime::support::EXTENT_BYTES,
};

use crate::c11_arena_append_crash::observation::observe_arena;

#[test]
fn killed_after_arena_data_settlement_replays_one_routed_extent() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_after_data_settlement(directory.path());
    let baseline = std::fs::read(directory.path().join("baseline-catalog")).unwrap();
    let catalog = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    assert_eq!(
        catalog, baseline,
        "the killed writer did not publish its root"
    );
    let unpublished_generation = generation(&catalog);
    observe_arena(&root, directory.path(), "killed");

    for _ in 0..2 {
        match worth_store_recovery_runtime::WorthStoreRecovery::recover(recovery_request(&root)) {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(_) => {}
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
                "ordinary arena append redo blocked: {:?}",
                block.evidence().planning_denial
            ),
            other => panic!("ordinary arena append redo did not recover: {other:?}"),
        }
    }
    let published = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    assert!(generation(&published) > unpublished_generation);
    let serving = open(&root);
    let rows = scan(&serving);
    assert_eq!(
        rows.len(),
        2,
        "durable WAL replays one append, not a duplicate"
    );
    assert!(rows.iter().any(|(_, payload)| payload == b"seed"));
    assert!(rows
        .iter()
        .any(|(_, payload)| payload.as_slice() == [43; EXTENT_BYTES]));
    serving.close();
    observe_arena(&root, directory.path(), "recovered");

    let reopened = open(&root);
    assert_eq!(
        scan(&reopened).len(),
        rows.len(),
        "a second fresh open adds no row"
    );
    let (_, placement, _) = configuration();
    let next = append(&reopened, placement, [178; 32], &[44; EXTENT_BYTES]);
    assert_eq!(next.persisted_records().len(), 1);
    let after = scan(&reopened);
    assert_eq!(after.len(), 3);
    for (identity, payload) in &rows {
        assert!(
            after
                .iter()
                .any(|(observed, bytes)| observed == identity && bytes == payload),
            "the next allocation preserves each original identity and payload"
        );
    }
    assert!(after
        .iter()
        .any(|(_, payload)| payload.as_slice() == [44; EXTENT_BYTES]));
    reopened.close();
    observe_arena(&root, directory.path(), "allocated-again");
}

#[test]
#[ignore = "spawned and killed after real arena frame settlement"]
fn arena_append_after_data_child() {
    let directory = std::path::PathBuf::from(std::env::var_os(MARKER).unwrap());
    let root = directory.join("store");
    let serving = initialize(&root);
    let (_, placement, _) = configuration();
    append(&serving, placement, [175; 32], b"seed");
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([176; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(checkpoint) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("genesis checkpoint must admit");
    };
    assert!(matches!(
        checkpoint.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    let original_catalog = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    std::fs::write(directory.join("baseline-catalog"), &original_catalog).unwrap();
    let gate = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
    );
    let witness_root = root.clone();
    std::thread::spawn(move || {
        assert!(
            gate.await_arrival(),
            "append did not reach the data-settled seam"
        );
        assert_eq!(
            std::fs::read(witness_root.join("families/records/bootstrap.catalog")).unwrap(),
            original_catalog,
            "root publication must not have started"
        );
        let arenas = std::fs::read_dir(witness_root.join("families/records/arenas"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(arenas.len(), 1);
        assert!(arenas[0].metadata().unwrap().len() >= EXTENT_BYTES as u64);
        std::fs::write(directory.join("reached"), b"1").unwrap();
    });
    append(&serving, placement, [177; 32], &[43; EXTENT_BYTES]);
    panic!("armed data-settled seam unexpectedly completed");
}
