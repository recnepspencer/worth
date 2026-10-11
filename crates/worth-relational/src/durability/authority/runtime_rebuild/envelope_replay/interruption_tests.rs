use std::sync::mpsc;
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
    create_entity(
        &ExecutionRequest::serial(&host),
        &source,
        "replay-interruption",
    );
    let plan = source
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    assert_eq!(plan.tail_log.len(), 1);
    let entry = &plan.tail_log[0];
    let bytes = rmp_serde::to_vec_named(
        &crate::durability::log::PersistedCanonicalCommit::from_positioned(
            entry.positioned().unwrap(),
        ),
    )
    .unwrap();
    (source, bytes)
}

fn plan_from_bytes(source: &RelationalRuntime, bytes: &[u8]) -> RecoveryPlan {
    let persisted: crate::durability::log::PersistedCanonicalCommit =
        rmp_serde::from_slice(bytes).unwrap();
    let mut plan = source
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    plan.tail_log = vec![persisted.readmit().unwrap()];
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
    assert_eq!(result.recovered_commits, 1);
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
        1
    );
}

#[test]
fn cancellation_at_an_effect_boundary_is_control_and_the_same_bytes_retry() {
    let (source, bytes) = fixture();
    let mut target = runtime_with_test_schema();
    let before = target.history().latest_commit();
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        cancellation.token(),
        None,
    );
    let (reached, receive) = mpsc::channel();
    let (resume, resumed) = mpsc::channel();
    let error = std::thread::scope(|scope| {
        scope.spawn(move || {
            receive
                .recv()
                .expect("replay finished without reaching the effect boundary");
            cancellation.cancel();
            resume.send(()).unwrap();
        });
        super::observation::with_observer(
            Box::new(move || {
                reached.send(()).unwrap();
                resumed.recv().unwrap();
            }),
            || {
                target.durability_recovery().recover(
                    &ExecutionRequest::serial(&host),
                    plan_from_bytes(&source, &bytes),
                )
            },
        )
        .unwrap_err()
    });
    assert_interruption(error, Cause::Cancelled);
    assert_eq!(target.history().latest_commit(), before);
    live_retry(&source, &bytes, &mut target);
}

#[test]
fn expired_replay_is_control_and_the_same_bytes_retry() {
    let (source, bytes) = fixture();
    let mut target = runtime_with_test_schema();
    let before = target.history().latest_commit();
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        cancellation.token(),
        Some(Instant::now() - Duration::from_secs(1)),
    );
    let error = target
        .durability_recovery()
        .recover(
            &ExecutionRequest::serial(&host),
            plan_from_bytes(&source, &bytes),
        )
        .unwrap_err();
    assert_interruption(error, Cause::DeadlineElapsed);
    assert_eq!(target.history().latest_commit(), before);
    live_retry(&source, &bytes, &mut target);
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
