//! Real native performance and linear repair custody under caller byte policy.
use super::*;
use application_installation::{
    WorthQueryCheckpointCaptureDenial as CaptureDenial,
    WorthQueryInMemoryApplicationDenial as InstallationDenial,
};
use std::{num::NonZeroUsize, sync::Mutex};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::runtime::{
    CancellationToken, ExecutionAllocationDenialKind as AllocationKind, LeaseDenial, LeaseRequest,
};

static SERIAL: Mutex<()> = Mutex::new(());

fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn request(bytes: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), bytes, 1),
        ),
        deadline: None,
        cancellation,
    }
}

fn author_record(
    writer: &mut application_installation::WorthQueryCheckpointMigrationWriter<
        '_,
        TemporalHostSchema,
    >,
    key: &str,
    value: u64,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    writer.bind_entity(
        primary_graph::WorthQueryApplicationEntitySeed::new(
            UnrelatedRecord::reference(),
            primary_graph::WorthQueryApplicationEntityKey::new(key).unwrap(),
        )
        .field(UnrelatedValueField::reference(), value),
    )
}

fn assert_target_record(
    checkpoint: application_installation::WorthQueryApplicationCheckpoint,
    value: u64,
) {
    let reopened = application_installation::in_memory_program_from_checkpoint(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
        checkpoint,
    )
    .expect("ordinary target admission authenticates the repaired native successor");
    let selected = reopened
        .on_branch(reopened.current_world())
        .select()
        .unwrap();
    let record = selected
        .resolve_entity(
            UnrelatedValueField::reference(),
            value,
            &request_scope(),
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the actual typed migrated value survives native recovery");
    assert_eq!(record.examined_candidate_count(), 1);
}

#[test]
fn checkpoint_transition_admission_refusal_retains_acknowledged_effects_for_explicit_repair() {
    let _serial = serial_guard();
    let (source, predecessor) = source();
    let lease = primary_graph::test_execution_authority()
        .request_lease(request(0, CancellationToken::new()))
        .unwrap();
    let mut calls = 0;
    let denial =
        application_installation::in_memory_rostered_program_from_checkpoint_with_transition(
            validated_program(),
            application_installation::WorthQueryApplicationProgramRoster::new(),
            TemporalHostSchema::declaration().unwrap(),
            configuration(),
            checkpoint::checkpoint_limits(),
            source,
            predecessor,
            Resources::bounded(512, 32, 4096).unwrap(),
            CapturePolicy::Execution(&lease),
            |writer, _| {
                calls += 1;
                author_record(writer, "allocation-repaired-record", 127)
            },
        )
        .err()
        .expect("zero caller payload budget refuses checkpoint capture after the effect");
    let InstallationDenial::CheckpointTransitionCaptureStopped(pending) = denial else {
        panic!("acknowledged publication must retain its exact repair custody: {denial:?}");
    };
    let Some(CaptureDenial::Allocation(denied)) = pending.capture_denial() else {
        panic!("exact native allocation cause must survive: {pending:?}");
    };
    assert_eq!(
        denied.kind(),
        AllocationKind::Lease(LeaseDenial::MemoryExhausted(
            worth_query_host::facade::runtime::MemoryLimitDenial {
                requested: denied.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    let native_quote = denied
        .requested_payload_bytes()
        .expect("checked real native payload quote");
    assert!(native_quote > 0);
    assert_eq!(
        calls, 1,
        "capture refusal follows actual authoring and acknowledgment"
    );
    let native_only = primary_graph::test_execution_authority()
        .request_lease(request(native_quote, CancellationToken::new()))
        .unwrap();
    let pending = pending
        .repair_to_checkpoint(CapturePolicy::Execution(&native_only))
        .expect_err("the final frame is admitted while native backing remains live");
    let Some(CaptureDenial::Allocation(denied)) = pending.capture_denial() else {
        panic!("exact final-frame allocation cause must survive: {pending:?}");
    };
    assert_eq!(
        denied.kind(),
        AllocationKind::Lease(LeaseDenial::MemoryExhausted(
            worth_query_host::facade::runtime::MemoryLimitDenial {
                requested: denied.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    let frame_quote = denied
        .requested_payload_bytes()
        .expect("checked real final-frame payload quote");
    assert!(frame_quote > native_quote);
    let released = native_only.reserve_memory(native_quote).unwrap();
    drop(released);
    drop(native_only);
    let repair = primary_graph::test_execution_authority()
        .request_lease(request(
            native_quote.checked_add(frame_quote).unwrap(),
            CancellationToken::new(),
        ))
        .unwrap();
    let checkpoint = pending
        .repair_to_checkpoint(CapturePolicy::Execution(&repair))
        .expect("a different explicit admitted policy captures the existing successor");
    assert_eq!(checkpoint.bytes().len() as u64, frame_quote);
    assert_eq!(checkpoint.charged_payload_bytes(), Some(frame_quote));
    assert_target_record(checkpoint, 127);
    assert_eq!(calls, 1, "repair cannot rerun authoring");
}

#[test]
fn checkpoint_transition_cancelled_repair_preserves_deferred_phase_and_clears_stale_capture_cause()
{
    let _serial = serial_guard();
    let (source, predecessor) = source();
    let mut calls = 0;
    let denial = transition(
        source,
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| {
            calls += 1;
            author_record(writer, "cancelled-repair-record", 131)?;
            writer.fail_next_durable_append_for_test();
            Ok(())
        },
    )
    .err()
    .expect("actual native append refusal retains unpublished deferred effects");
    let InstallationDenial::CheckpointTransitionDeferred(pending) = denial else {
        panic!("expected genuine deferred native custody: {denial:?}");
    };
    assert!(pending.capture_denial().is_none());
    // Early policy cancellation must not consume either the deferred phase or
    // the actual next native append fault, which is exercised below.
    pending.fail_next_durable_append_for_test();
    let cancellation = worth_query_host::facade::runtime::CancellationSource::new();
    let lease = primary_graph::test_execution_authority()
        .request_lease(request(0, cancellation.token()))
        .unwrap();
    cancellation.cancel();
    let pending = pending
        .repair_to_checkpoint(CapturePolicy::Execution(&lease))
        .expect_err("the explicit stopped policy cannot settle or capture");
    let Some(CaptureDenial::Allocation(denied)) = pending.capture_denial() else {
        panic!("early stop must retain its exact current-attempt cause: {pending:?}");
    };
    assert_eq!(denied.kind(), AllocationKind::Cancelled);
    assert_eq!(denied.requested_payload_bytes(), None);
    let pending = pending
        .repair_to_checkpoint(CapturePolicy::SystemAllocation)
        .expect_err(
            "unconsumed injected append failure proves deferred custody survived cancellation",
        );
    assert!(
        pending.capture_denial().is_none(),
        "the later actual native settlement failure cannot retain an obsolete capture cause"
    );
    let checkpoint = pending
        .repair_to_checkpoint(CapturePolicy::SystemAllocation)
        .expect("native repair settles the same performed transition without authoring again");
    assert_eq!(checkpoint.charged_payload_bytes(), None);
    assert_target_record(checkpoint, 131);
    assert_eq!(calls, 1);
}
