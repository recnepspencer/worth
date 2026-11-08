//! Custody-capable checkpoint framing, independent of the unchanged body grammar.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{RecoveryFailureClass, RelationalNativeCheckpoint, RelationalRecoveryStop},
    history::BranchId,
};
use worth_testkit_relational::maintenance::relation_laws::{request, World};

#[test]
fn custody_checkpoint_revision_reads_legacy_and_refuses_unknown_versions() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let world = World::new(&execution, 1);
    let current = world
        .runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    // Literal protocol versions are interoperability data, not a cost golden.
    let body = current
        .bytes()
        .strip_prefix(b"WORTH-REL-CHK20.1\n")
        .expect("new custody-capable checkpoints must fence CHK20-only readers");
    let mut legacy = b"WORTH-REL-CHK20\n".to_vec();
    legacy.extend_from_slice(body);
    let legacy = RelationalNativeCheckpoint::from_untrusted_bytes(legacy);
    let now = RelationalNativeCheckpoint::inspect_bytes(&execution, current.bytes()).unwrap();
    let old = RelationalNativeCheckpoint::inspect_bytes(&execution, legacy.bytes()).unwrap();
    assert_eq!(now.model, old.model, "only the capability marker changes");
    assert_eq!(now.live_records, old.live_records);
    for checkpoint in [&current, &legacy] {
        let mut restored = World::empty_runtime(0);
        restored
            .durability_recovery()
            .restore_native_checkpoint(&execution, checkpoint)
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
    }
    let mut future = b"WORTH-REL-CHK20.2\n".to_vec();
    future.extend_from_slice(body);
    let inspected = RelationalNativeCheckpoint::inspect_bytes(&execution, &future).unwrap_err();
    assert_unsupported(inspected);
    let mut target = World::empty_runtime(0);
    let before = target
        .branch_reference_state(&BranchId("main".into()))
        .unwrap();
    let stop = target
        .durability_recovery()
        .restore_native_checkpoint(
            &execution,
            &RelationalNativeCheckpoint::from_untrusted_bytes(future),
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

fn assert_unsupported(stop: RelationalRecoveryStop) {
    match stop {
        RelationalRecoveryStop::Durability(failure) => assert_eq!(
            failure.class,
            RecoveryFailureClass::UnsupportedLegacySemantics
        ),
        other => panic!("version must refuse before parsing or loading rows: {other:?}"),
    }
}
