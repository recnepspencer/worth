//! A valid selected frame still has to belong to its indexed original subject.
use super::*;
use crate::durability::canonical_commit_artifact::{
    borrowed_payload::BorrowedPayload,
    RelationalCanonicalCommitArtifactEncodeDenial as ExportDenied,
};
use crate::durability::checkpoint_history::inventory::CanonicalCheckpointMember;
use crate::durability::data::RelationalNativeCheckpoint;

fn crossed_runtime(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::runtime::RelationalRuntime,
    Arc<MemberSource>,
    crate::publication::patch::data::PatchStreamPosition,
    RelationalDurableMemberLocator,
) {
    let (original, source, _) = fixture_with_history(execution, false, 6);
    source.damaged.store(false, Ordering::Relaxed);
    let checkpoint = original.checkpoint().unwrap();
    let bytes = checkpoint.checkpoint().bytes();
    let magic = crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC;
    let rows = BorrowedPayload(bytes.strip_prefix(magic).unwrap())
        .field("manifest", execution)
        .unwrap()
        .field("canonical_members", execution)
        .unwrap();
    let mut rewritten: Vec<CanonicalCheckpointMember> = rmp_serde::from_slice(rows.0).unwrap();
    let position = rewritten[0].position;
    let foreign = rewritten[1].member;
    let original_first = rewritten[0].member;
    rewritten[0].member = foreign;
    rewritten[1].member = original_first;
    let offset = rows.0.as_ptr() as usize - bytes.as_ptr() as usize;
    let mut corrupt = bytes[..offset].to_vec();
    corrupt.extend(rmp_serde::to_vec_named(&rewritten).unwrap());
    corrupt.extend_from_slice(&bytes[offset + rows.0.len()..]);
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered(),
            checkpoint.covered_transitions(),
            RelationalNativeCheckpoint::from_untrusted_bytes(corrupt),
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut runtime = durable_runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    (runtime, source, position, foreign)
}

#[test]
fn strict_original_capture_keeps_work_stop_precedence_over_retained_damage() {
    use worth_execution::{ExecutionWorkCeiling, MapKernelStop, WorkCeilingDenial};
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (runtime, source, position, _) = crossed_runtime(execution);
    let first = runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .next()
        .unwrap();
    assert!(
        matches!(first, Err(ExportDenied::StorageUnavailable(failure)) if failure.cause == Cause::IntegrityMismatch)
    );
    let receipt = runtime
        .history()
        .immutable_commit_receipt_at_patch_stream_position(position)
        .unwrap();
    let cell = runtime
        .history
        .commit_artifact(receipt.commit_id)
        .unwrap()
        .envelope_residency();
    let reads = source.canonical_member_reads.lock().unwrap().len();
    let (native, _) = execution
        .run(ExecutionWorkCeiling::new(Some(0)), |_| {
            execution.run_native_with_checkpoint(|context| {
                assert_eq!(context.checkpoint(1), Err(MapKernelStop::WorkCeiling));
                let stopped = execution.consult().unwrap_err();
                assert_eq!(
                    stopped,
                    WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling)
                );
                match cell.require_original_artifact(execution, None) {
                    Err(RelationalRecoveryStop::Request(cause)) => {
                        assert_eq!(cause, stopped.into())
                    }
                    other => panic!("the invoking work stop precedes retained damage: {other:?}"),
                }
            })
        })
        .unwrap();
    assert!(
        native.is_err(),
        "the same native owner retains the swallowed work stop"
    );
    assert_eq!(source.canonical_member_reads.lock().unwrap().len(), reads);
}
