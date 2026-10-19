use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Instant;

use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

use crate::durability::data::{
    RecoveryFailureClass, RecoveryPlan, RecoveryVerificationMode, RelationalRecoveryStop,
};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::runtime::RelationalRuntime;
use crate::tests::support::{create_entity, runtime_with_test_schema};

fn fixture() -> (RelationalRuntime, Vec<u8>) {
    let source = runtime_with_test_schema();
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    for name in ["replay-first", "replay-second", "replay-third"] {
        create_entity(&ExecutionRequest::serial(&host), &source, name);
    }
    let plan = source
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    assert_eq!(plan.tail_log.len(), 3);
    let persisted: Vec<_> = plan
        .tail_log
        .iter()
        .map(|entry| {
            crate::durability::log::PersistedCanonicalCommit::from_positioned(entry.positioned())
        })
        .collect();
    let bytes = rmp_serde::to_vec_named(&persisted).unwrap();
    (source, bytes)
}

fn plan_from_bytes(source: &RelationalRuntime, bytes: &[u8]) -> RecoveryPlan {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let persisted: Vec<crate::durability::log::PersistedCanonicalCommit> =
        rmp_serde::from_slice(bytes).unwrap();
    let mut plan = source
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.tail_log = persisted
        .into_iter()
        .map(|record| record.readmit().unwrap())
        .collect();
    plan
}

fn assert_interruption(error: RelationalRecoveryStop, cause: Cause) {
    assert_eq!(error, RelationalRecoveryStop::Request(cause));
}

#[test]
fn verification_replay_returns_control_outside_its_serialized_outcome() {
    let (source, _bytes) = fixture();
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let request = crate::replay::data::RelationalReplayRequest {
        commit_id: source.history().latest_commit().unwrap().commit_id,
        branch_id: crate::history::data::BranchId("main".into()),
        execution_mode: crate::replay::data::ReplayExecutionMode::SerialDeterministic,
        verification_mode: crate::replay::data::ReplayVerificationMode::NormalRecoveryVerification,
    };
    let error = source
        .replay_authority()
        .replay_commit(request, &ExecutionRequest::serial(&host))
        .unwrap_err();
    assert_interruption(error, Cause::Cancelled);
}

fn live_retry(source: &RelationalRuntime, bytes: &[u8], target: &mut RelationalRuntime) {
    let host = crate::test_execution_policy::serial_request();
    let result = target
        .durability_recovery()
        .recover(
            &ExecutionRequest::serial(&host),
            plan_from_bytes(source, bytes),
        )
        .unwrap();
    assert_eq!(result.recovered_commits, 3);
    let mut expected = runtime_with_test_schema();
    expected
        .durability_recovery()
        .recover(
            &ExecutionRequest::serial(&host),
            plan_from_bytes(source, bytes),
        )
        .unwrap();
    let actual = comparable_recovered_state(target);
    let expected = comparable_recovered_state(&expected);
    assert!(
        actual == expected,
        "retry changed whole state: {:?}",
        actual
            .lines()
            .zip(expected.lines())
            .enumerate()
            .find(|(_, (a, b))| a != b)
            .map(|(line, pair)| (
                line,
                pair,
                actual
                    .lines()
                    .skip(line.saturating_sub(8))
                    .take(16)
                    .collect::<Vec<_>>()
            ))
    );
    assert_eq!(
        target.history().latest_commit(),
        source.history().latest_commit()
    );
    let snapshot = crate::tests::support::snapshot_for_owner_branch(
        target,
        &crate::history::data::BranchId("main".into()),
    );
    assert_eq!(
        target
            .read_truth()
            .read_snapshot(&snapshot, &ExecutionRequest::serial(&host))
            .unwrap()
            .unwrap()
            .entities()
            .len(),
        3
    );
}

// Debug includes the owned runtime's rows, schema, indexes, history, diagnostics,
// resource registrations and counters, not just its latest commit identity.
fn observable_state(target: &RelationalRuntime) -> String {
    format!("{target:#?}")
}

fn interrupted_mid_range() {
    use crate::authority::commit::pipeline::{stage_observation, CommitStage};
    let (source, bytes) = fixture();
    let mut target = runtime_with_test_schema();
    let before = observable_state(&target);
    let cancellation = CancellationSource::new();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let applied = Arc::new(AtomicUsize::new(0));
    let observed = applied.clone();
    let mut validations = 0;
    let mut first_published = false;
    let error = stage_observation::with_observer(
        Box::new(move |stage| {
            if stage == CommitStage::Publication {
                first_published = true;
            }
            if stage == CommitStage::Validation {
                validations += 1;
                if validations == 2 {
                    // The previous Publication consult completed before this record.
                    assert!(first_published, "first record never reached publication");
                    observed.fetch_add(1, Ordering::SeqCst);
                    cancellation.cancel();
                }
            }
        }),
        || {
            target.durability_recovery().recover(
                &ExecutionRequest::serial(&host),
                plan_from_bytes(&source, &bytes),
            )
        },
    )
    .unwrap_err();
    assert_eq!(
        applied.load(Ordering::SeqCst),
        1,
        "replay stopped before the second record"
    );
    assert_interruption(error, Cause::Cancelled);
    assert_eq!(
        observable_state(&target),
        before,
        "interrupted replay changed its target"
    );
    live_retry(&source, &bytes, &mut target);
}

#[test]
fn cancellation_after_an_applied_record_leaves_the_whole_target_unchanged() {
    interrupted_mid_range();
}

#[test]
fn recovery_entry_stops_change_no_record_or_diagnostic() {
    use crate::authority::commit::pipeline::stage_observation;
    for cause in [Cause::Cancelled, Cause::DeadlineElapsed] {
        let (source, bytes) = fixture();
        let mut target = runtime_with_test_schema();
        let before = observable_state(&target);
        let cancellation = CancellationSource::new();
        if cause == Cause::Cancelled {
            cancellation.cancel();
        }
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(None),
            cancellation.token(),
            (cause == Cause::DeadlineElapsed).then(Instant::now),
        );
        let stages = Arc::new(AtomicUsize::new(0));
        let observed = stages.clone();
        let mut plan = plan_from_bytes(&source, &bytes);
        plan.authority_continuity.verification_outcome =
            crate::durability::data::RecoveryVerificationOutcome::Rejected {
                layer: crate::replay::data::ReplayVerificationLayer::DigestParity,
                detail: "entry-stop control rejects recovery admission".into(),
            };
        // Rejected marks diagnostic posture; parity drift makes admission refuse.
        plan.authority_continuity.schema_parity =
            crate::durability::data::RecoveryAuthorityParity::Drift;
        let result = stage_observation::with_observer(
            Box::new(move |_| {
                observed.fetch_add(1, Ordering::SeqCst);
            }),
            || {
                target
                    .durability_recovery()
                    .recover(&ExecutionRequest::serial(&host), plan)
            },
        );
        assert_eq!(result.unwrap_err(), RelationalRecoveryStop::Request(cause));
        assert_eq!(
            stages.load(Ordering::SeqCst),
            0,
            "{cause:?}: entry stop applied no record"
        );
        assert_eq!(
            observable_state(&target),
            before,
            "{cause:?}: entry stop changed target or diagnostics"
        );
    }
}

#[test]
fn cancelled_checkpoint_restore_consults_before_decoding_corrupt_bytes() {
    let mut target = runtime_with_test_schema();
    let before = observable_state(&target);
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let corrupt =
        crate::durability::data::RelationalNativeCheckpoint::from_untrusted_bytes(vec![0xff]);
    assert_eq!(
        target
            .durability_recovery()
            .restore_native_checkpoint(&ExecutionRequest::serial(&host), &corrupt,)
            .unwrap_err(),
        RelationalRecoveryStop::Request(Cause::Cancelled),
    );
    assert_eq!(
        observable_state(&target),
        before,
        "entry stop changed target or diagnostics"
    );
}

#[test]
fn corrupt_replay_keeps_its_durability_failure() {
    let (source, bytes) = fixture();
    let plan = plan_from_bytes(&source, &bytes);
    let mut envelope = plan.tail_log[0].envelope().clone();
    // A merge without its required execution authority is corrupt evidence.
    envelope
        .merge_parent_branches
        .push(envelope.branch_context.clone());
    let mut target = runtime_with_test_schema();
    let host = crate::test_execution_policy::serial_request();
    let error = super::effect::replay_merge_commit(
        &ExecutionRequest::serial(&host),
        &mut target,
        &envelope,
        plan.tail_log[0].position(),
    )
    .unwrap_err();
    match error {
        RelationalRecoveryStop::Durability(error) => {
            assert_eq!(error.class, RecoveryFailureClass::ReplayFailure)
        }
        RelationalRecoveryStop::Request(cause) => {
            panic!("corrupt evidence became request control: {cause:?}")
        }
        RelationalRecoveryStop::Allocation(cause) => {
            panic!("corrupt evidence became allocation failure: {cause:?}")
        }
    }
}

mod recovered_state_comparison;
use recovered_state_comparison::comparable_recovered_state;
