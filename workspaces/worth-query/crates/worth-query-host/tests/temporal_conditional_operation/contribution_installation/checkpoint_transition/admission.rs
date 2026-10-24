//! Real native performance and linear repair custody under caller byte policy.
use super::*;
use application_installation::{
    WorthQueryCheckpointCaptureDenial as CaptureDenial,
    WorthQueryInMemoryApplicationDenial as InstallationDenial,
};
use std::sync::Mutex;
use worth_execution::ExecutionRequest;
use worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::runtime::{
    CancellationToken, ExecutionAllocationDenialKind as AllocationKind, LeaseDenial,
};

static SERIAL: Mutex<()> = Mutex::new(());

fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn funding_lease() -> worth_execution::ExecutionResourceLease<'static> {
    primary_graph::query_test_execution_authority_for_test()
        .request_lease(super::original_request::request(CancellationToken::new()))
        .unwrap()
}

/// The caller lease, `ancestor` controlled levels above the charging worker,
/// has only `admitted` of `requested` left.
fn exhausted_lease(requested: u64, admitted: u64, ancestor: u32) -> AllocationKind {
    AllocationKind::Lease(LeaseDenial::MemoryExhausted(
        worth_query_host::facade::runtime::MemoryLimitDenial::Capacity {
            requested,
            admitted,
            level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor },
        },
    ))
}

/// One caller lease funds the authoring, the bootstrap commit and the capture.
/// To prove ACK precedes capture exhaustion the hook leaves the commit exactly
/// its quote, learned from the commit's own refusal (MENTALITY §7): each refused
/// run adds `requested - admitted`. Returns the final outcome and that quote.
fn until_committed<T>(
    mut attempt: impl FnMut(u64) -> Result<T, InstallationDenial>,
) -> (Result<T, InstallationDenial>, u64) {
    use primary_graph::{
        WorthQueryPrimaryGraphInstallationDenialKind as Kind,
        WorthQueryProviderSessionDenialKind as Session,
    };
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    let mut keep = 0;
    let mut quotes = Vec::new();
    // A bound, not a count: learning ends when the commit commits.
    for _ in 0..64 {
        match attempt(keep) {
            Err(InstallationDenial::Graph(cause))
                if cause.subject().contains("bootstrap transaction") =>
            {
                let Kind::ExecutionDenied {
                    kind:
                        Session::ExecutionResource {
                            denial: Resource::MemoryLimit { requested, admitted, .. },
                            policy_ancestor: Some(0),
                            ..
                        },
                } = cause.kind()
                else {
                    return (Err(InstallationDenial::Graph(cause)), keep);
                };
                quotes.push((requested, admitted));
                keep += requested - admitted;
            }
            outcome => return (outcome, keep),
        }
    }
    panic!("the bootstrap commit still refused after 64 learned quotes: {quotes:?}")
}

pub(super) fn author_record(
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

pub(super) fn assert_target_record(
    checkpoint: application_installation::WorthQueryApplicationCheckpoint,
    value: u64,
) {
    let reopened = application_installation::in_memory_program_from_checkpoint(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint::checkpoint_limits(),
        checkpoint,
    )
    .expect("ordinary target admission authenticates the repaired native successor");
    let selected = reopened
        .with_host_advancement(|host_phase| {
            reopened
                .on_branch(reopened.current_world())
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let record = reopened
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    UnrelatedValueField::reference(),
                    value,
                    &request_scope(),
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("the actual typed migrated value survives native recovery")
        })
        .expect("installed host call opens");
    assert_eq!(record.examined_candidate_count(), 1);
}

#[test]
fn checkpoint_transition_admission_refusal_retains_acknowledged_effects_for_explicit_repair() {
    let _serial = serial_guard();
    let (source, predecessor) = source();
    let lease = funding_lease();
    let mut calls = 0;
    let (denial, keep) = until_committed(|keep| {
        calls = 0;
        let mut exhausted = None;
        let outcome =
            application_installation::in_memory_rostered_program_from_checkpoint_with_transition(
                validated_program(),
                application_installation::WorthQueryApplicationProgramRoster::new(),
                TemporalHostSchema::declaration().unwrap(),
                configuration(),
                ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
                checkpoint::checkpoint_limits(),
                source.clone(),
                predecessor.clone(),
                Resources::bounded(512, 32, 4096).unwrap(),
                CapturePolicy::Execution(&lease),
                |writer, _| {
                    calls += 1;
                    author_record(writer, "allocation-repaired-record", 127)?;
                    // The lease funds the whole call; once the effect is
                    // authored it keeps only the commit's quote.
                    let room = lease.available_memory_bytes().expect("finite caller lease");
                    exhausted = Some(lease.reserve_memory(room - keep).unwrap());
                    Ok(())
                },
            );
        drop(exhausted);
        outcome
    });
    let denial = denial
        .err()
        .expect("an exhausted caller lease refuses checkpoint capture after the effect");
    let InstallationDenial::CheckpointTransitionCaptureStopped(pending) = denial else {
        panic!("acknowledged publication must retain its exact repair custody: {denial:?}");
    };
    let Some(CaptureDenial::Allocation(denied)) = pending.capture_denial() else {
        panic!("exact native allocation cause must survive: {pending:?}");
    };
    let native_quote = denied
        .requested_payload_bytes()
        .expect("checked real native payload quote");
    assert!(native_quote > 0);
    // The committed transaction returned its quote; the capture charges the
    // call's controlled child of the lease.
    assert_eq!(denied.kind(), exhausted_lease(native_quote, keep, 1));
    assert_eq!(
        calls, 1,
        "capture refusal follows actual authoring and acknowledgment"
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
    let native_only = funding_lease();
    let pending =
        with_bootstrap_advancement_from_request(ExecutionRequest::leased(&native_only), |phase| {
            // Leave the repair exactly the native backing's room.
            let room = native_only.available_memory_bytes().unwrap();
            let _held = native_only.reserve_memory(room - native_quote).unwrap();
            pending
                .repair_to_checkpoint(&phase, CapturePolicy::Execution(&native_only))
                .expect_err("the final frame is refused while native backing remains live")
        })
        .expect("the funding lease opens the repair");
    let Some(CaptureDenial::Allocation(denied)) = pending.capture_denial() else {
        panic!("exact final-frame allocation cause must survive: {pending:?}");
    };
    let frame_quote = denied
        .requested_payload_bytes()
        .expect("checked real final-frame payload quote");
    assert!(frame_quote > native_quote);
    assert_eq!(denied.kind(), exhausted_lease(frame_quote, 0, 2));
    assert_eq!(native_only.charged_memory_bytes(), 0);
    let repair = funding_lease();
    let checkpoint = pending
        .repair_to_checkpoint_with_request(ExecutionRequest::leased(&repair))
        .expect("a different explicit admitted policy captures the existing successor");
    assert_eq!(checkpoint.bytes().len() as u64, frame_quote);
    assert_eq!(checkpoint.charged_payload_bytes(), Some(frame_quote));
    assert_eq!(repair.charged_memory_bytes(), frame_quote);
    assert_target_record(checkpoint, 127);
    assert_eq!(repair.charged_memory_bytes(), 0);
    assert_eq!(calls, 1, "repair cannot rerun authoring");
}

#[test]
fn checkpoint_transition_cancelled_repair_preserves_deferred_phase_and_clears_stale_capture_cause()
{
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
            writer.fail_next_durable_append_for_test(request_execution);
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
    pending.fail_next_durable_append_for_test(request_execution);
    // The caller lease funds the repair it opens, so a stopped caller
    // request refuses at entry and returns the same deferred capsule.
    let cancellation = worth_query_host::facade::runtime::CancellationSource::new();
    let lease = primary_graph::query_test_execution_authority_for_test()
        .request_lease(super::original_request::request(cancellation.token()))
        .unwrap();
    cancellation.cancel();
    let pending = pending
        .repair_to_checkpoint_with_request(ExecutionRequest::leased(&lease))
        .expect_err("the stopped caller request cannot settle or capture");
    let Some(CaptureDenial::Execution(cause)) = pending.capture_denial() else {
        panic!("early stop must retain its exact current-attempt cause: {pending:?}");
    };
    super::original_request::assert_cancelled(cause);
    let pending = crate::test_request_policy::with_bootstrap(|bootstrap_phase| {
        pending
            .repair_to_checkpoint(&bootstrap_phase, CapturePolicy::SystemAllocation)
            .expect_err(
                "unconsumed injected append failure proves deferred custody survived cancellation",
            )
    });
    assert!(
        pending.capture_denial().is_none(),
        "the later actual native settlement failure cannot retain an obsolete capture cause"
    );
    let checkpoint = crate::test_request_policy::with_bootstrap(|bootstrap_phase| {
        pending
            .repair_to_checkpoint(&bootstrap_phase, CapturePolicy::SystemAllocation)
            .expect("native repair settles the same performed transition without authoring again")
    });
    assert_eq!(checkpoint.charged_payload_bytes(), None);
    assert_target_record(checkpoint, 131);
    assert_eq!(calls, 1);
}

#[test]
fn original_request_deferred_repair_returns_same_custody_on_entry_and_work_stops() {
    use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
    let _serial = serial_guard();
    let (source, predecessor) = source();
    let lease = primary_graph::query_test_execution_authority_for_test()
        .request_lease(super::original_request::request(CancellationToken::new()))
        .unwrap();
    let original = ExecutionRequest::leased(&lease);
    let mut calls = 0;
    let denial =
        super::original_request::transition_original(original, source, predecessor, |writer, _| {
            calls += 1;
            author_record(writer, "original-deferred-record", 139)?;
            // Actual owner control remains at this existing allowlisted caller.
            writer.fail_next_durable_append_for_test(&original);
            Ok(())
        })
        .err()
        .expect("native append fault must return genuine deferred custody");
    let InstallationDenial::CheckpointTransitionDeferred(pending) = denial else {
        panic!("expected native deferred custody: {denial:?}");
    };
    let cancel = worth_query_host::facade::runtime::CancellationSource::new();
    let cancelled = primary_graph::query_test_execution_authority_for_test()
        .request_lease(super::original_request::request(cancel.token()))
        .unwrap();
    cancel.cancel();
    let pending = pending
        .repair_to_checkpoint_with_request(ExecutionRequest::leased(&cancelled))
        .expect_err("opening refusal must return the same deferred capsule");
    let Some(CaptureDenial::Execution(cause)) = pending.capture_denial() else {
        panic!("typed original cancellation retained");
    };
    super::original_request::assert_cancelled(cause);
    let exhausted = primary_graph::query_test_execution_authority_for_test()
        .request_lease(super::original_request::request(CancellationToken::new()))
        .unwrap();
    let execution = ExecutionRequest::leased(&exhausted);
    let (pending, _) = ExecutionWorkCeiling::new(Some(1024))
        .run_with_checkpoint(Some(&exhausted), None, |context| {
            let remaining = context
                .remaining_work()
                .expect("this native fixture installs a finite work ceiling");
            context.checkpoint_cost(remaining, remaining).unwrap();
            assert!(context.checkpoint_cost(1, 1).is_err());
            pending
                .repair_to_checkpoint_with_request(execution)
                .expect_err("the original native work refusal cannot be reopened")
        })
        .unwrap();
    let Some(CaptureDenial::Execution(cause)) = pending.capture_denial() else {
        panic!("typed original work stop retained");
    };
    super::original_request::assert_work(cause);
    let fresh = primary_graph::query_test_execution_authority_for_test()
        .request_lease(super::original_request::request(CancellationToken::new()))
        .unwrap();
    let checkpoint = pending
        .repair_to_checkpoint_with_request(ExecutionRequest::leased(&fresh))
        .expect("fresh funding repairs the original performed transition");
    assert!(checkpoint.charged_payload_bytes().is_some());
    assert_eq!(calls, 1, "repair never reruns the author callback");
    assert_target_record(checkpoint, 139);
}
