use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

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
    for name in ["replay-first", "replay-second", "replay-third"] {
        create_entity(&ExecutionRequest::serial(&host), &source, name);
    }
    let plan = source
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    assert_eq!(plan.tail_log.len(), 3);
    let persisted: Vec<_> = plan
        .tail_log
        .iter()
        .map(|entry| {
            crate::durability::log::PersistedCanonicalCommit::from_positioned(
                entry.positioned().unwrap(),
            )
        })
        .collect();
    let bytes = rmp_serde::to_vec_named(&persisted).unwrap();
    (source, bytes)
}

fn plan_from_bytes(source: &RelationalRuntime, bytes: &[u8]) -> RecoveryPlan {
    let persisted: Vec<crate::durability::log::PersistedCanonicalCommit> =
        rmp_serde::from_slice(bytes).unwrap();
    let mut plan = source
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
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
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        cancellation.token(),
        None,
    );
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

fn interrupted_mid_range(expire: bool) {
    use crate::authority::commit::pipeline::{stage_observation, CommitStage};
    let (source, bytes) = fixture();
    let mut target = runtime_with_test_schema();
    let before = observable_state(&target);
    let cancellation = CancellationSource::new();
    // Expiry is placed at the stage boundary, never before the host call.
    let deadline = expire.then(|| Instant::now() + Duration::from_secs(2));
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        cancellation.token(),
        deadline,
    );
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
                    if let Some(deadline) = deadline {
                        while Instant::now() < deadline {
                            std::thread::yield_now();
                        }
                    } else {
                        cancellation.cancel();
                    }
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
    assert_interruption(
        error,
        if expire {
            Cause::DeadlineElapsed
        } else {
            Cause::Cancelled
        },
    );
    assert_eq!(
        observable_state(&target),
        before,
        "interrupted replay changed its target"
    );
    live_retry(&source, &bytes, &mut target);
}

#[test]
fn cancellation_after_an_applied_record_leaves_the_whole_target_unchanged() {
    interrupted_mid_range(false);
}

#[test]
fn expiry_after_an_applied_record_leaves_the_whole_target_unchanged() {
    interrupted_mid_range(true);
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
    }
}
