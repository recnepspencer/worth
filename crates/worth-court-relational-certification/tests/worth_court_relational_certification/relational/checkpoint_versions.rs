//! Custody-capable checkpoint framing, independent of the unchanged body grammar.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{RecoveryFailureClass, RelationalNativeCheckpoint, RelationalRecoveryStop},
    history::BranchId,
};
use worth_testkit_relational::maintenance::relation_laws::{request, World};

#[test]
fn current_checkpoint_preserves_meaning_and_retired_or_unknown_versions_refuse() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let world = World::new(&execution, 1);
    let current = world
        .runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    // Protocol versions are interoperability data, not a measured cost golden.
    let body = current
        .bytes()
        .strip_prefix(b"WORTH-REL-CHK22\n")
        .expect("only the current combined checkpoint grammar is emitted");
    let now = RelationalNativeCheckpoint::inspect_bytes(&execution, current.bytes()).unwrap();
    let mut restored = World::empty_runtime(0);
    restored
        .durability_recovery()
        .restore_native_checkpoint(&execution, &current)
        .unwrap();
    assert_eq!(
        restored
            .branch_reference_state(&BranchId("main".into()))
            .unwrap()
            .truth_version(),
        world
            .runtime
            .branch_reference_state(&BranchId("main".into()))
            .unwrap()
            .truth_version()
    );
    let recaptured = restored
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let restored_meaning =
        RelationalNativeCheckpoint::inspect_bytes(&execution, recaptured.bytes()).unwrap();
    assert!(now
        .model_matches_rebound_runtime(
            &restored_meaning,
            &world
                .runtime
                .branch_identity(&BranchId("main".into()))
                .unwrap(),
            &restored.branch_identity(&BranchId("main".into())).unwrap(),
        )
        .unwrap());
    assert_eq!(now.live_records, restored_meaning.live_records);
    for prefix in [
        b"WORTH-REL-CHK18\n".as_slice(),
        b"WORTH-REL-CHK20\n".as_slice(),
        b"WORTH-REL-CHK20.1\n".as_slice(),
        b"WORTH-REL-CHK21\n".as_slice(),
        b"WORTH-REL-CHK23\n".as_slice(),
    ] {
        let mut bytes = prefix.to_vec();
        bytes.extend_from_slice(body);
        assert_unsupported(
            RelationalNativeCheckpoint::inspect_bytes(&execution, &bytes).unwrap_err(),
        );
        let mut target = World::empty_runtime(0);
        let before = target
            .branch_reference_state(&BranchId("main".into()))
            .unwrap();
        let stop = target
            .durability_recovery()
            .restore_native_checkpoint(
                &execution,
                &RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
            )
            .unwrap_err();
        assert_unsupported(stop);
        assert_eq!(
            before,
            target
                .branch_reference_state(&BranchId("main".into()))
                .unwrap()
        );
    }
}

fn assert_unsupported(stop: RelationalRecoveryStop) {
    match stop {
        RelationalRecoveryStop::Durability(failure) => assert_eq!(
            failure.class,
            RecoveryFailureClass::UnsupportedLegacySemantics
        ),
        other => panic!("version must refuse before parsing or loading rows: {other:?}"),
    }
}
