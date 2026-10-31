//! Real published results, original Native release cells and retained World progress.
use super::*;
use crate::branch::reference_test_fixture::{real_fixture, RealReferenceFixture};
use crate::facade::*;
use worth_execution::{CancellationSource, ExecutionRequest};

type Owner = RuntimeWorldOwner<(), (), (), (), ()>;
struct Clock;
impl RuntimeWorldClockSource for Clock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(0)
    }
}
fn published() -> (
    RealReferenceFixture,
    Owner,
    Arc<CommitResult>,
    CompositeOwnerExecutionResults,
) {
    let mut fixture = real_fixture(12, 12);
    let inputs = fixture.owner_inputs(
        RuntimeWorldBudgets::without_policy_limits(),
        RuntimeWorldClock::from_source(Clock),
    );
    let owner = RuntimeWorldOwner::builder()
        .with_execution_policy(inputs.execution.request_policy())
        .with_bridge_correspondence(inputs.bridge)
        .with_relational_services(inputs.relational)
        .with_signal_services(inputs.signal)
        .with_signal_definition_publication(inputs.signal_definition_publication)
        .with_budgets(inputs.budgets)
        .with_clock(inputs.clock)
        .build()
        .unwrap();
    let expected = match owner
        .lifecycle_port()
        .bootstrap_root(fixture.bootstrap_intent())
        .unwrap()
    {
        RuntimeWorldBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("real bootstrap: {other:?}"),
    };
    let policy = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let cancel = RuntimeWorldCancellationSource::new();
    let prepared = owner
        .publication_port()
        .prepare_without_signal(
            expected,
            CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                .with_prepared_relational_candidate(
                    fixture.prepare_relational_owner_candidate(&request, "actual-result-cleanup"),
                ),
            &cancel.token(),
            None,
        )
        .unwrap();
    let performed =
        match owner
            .publication_port()
            .execute_without_signal(request, prepared, &cancel.token())
        {
            RuntimeWorldPublicationOutcome::Performed(value) => value,
            other => panic!("real publication: {other:?}"),
        };
    let facts = performed.component_results().evidence_image();
    let result = facts.retain_relational_commit_result().unwrap();
    drop(performed.consume());
    let report = owner.lifecycle_port().close(&request, &owner).unwrap();
    assert!(report.retained_records().is_empty());
    (fixture, owner, result, facts)
}
fn drive(
    task: &mut RuntimeWorldResultSnapshotCleanup,
    request: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
) -> RuntimeWorldResultSnapshotCleanupReport {
    loop {
        match task.advance(request, native).unwrap() {
            RuntimeWorldResultSnapshotCleanupProgress::Advanced => {}
            RuntimeWorldResultSnapshotCleanupProgress::Complete(report) => return report,
        }
    }
}
#[test]
fn result_cleanup_initialization_stop_and_current_refusals_keep_original_custody() {
    let (fixture, owner, result, facts) = published();
    let cancel = CancellationSource::new();
    let stopped = crate::test_execution_policy::serial_request().with_cancellation(cancel.token());
    cancel.cancel();
    assert!(matches!(
        owner.result_snapshot_cleanup_after_close(&ExecutionRequest::serial(&stopped)),
        Err(RuntimeWorldResultSnapshotCleanupDenial::Stopped(_))
    ));
    let policy = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let held = owner.root.state.close.lock().unwrap();
    assert!(matches!(
        owner.result_snapshot_cleanup_after_close(&request),
        Err(RuntimeWorldResultSnapshotCleanupDenial::Busy)
    ));
    drop(held);
    let mut task = owner.result_snapshot_cleanup_after_close(&request).unwrap();
    assert!(matches!(
        task.advance(&request, fixture.relational_runtime())
            .unwrap(),
        RuntimeWorldResultSnapshotCleanupProgress::Advanced
    ));
    assert!(
        task.current.is_none(),
        "first exact root slot has no Native result"
    );
    task.advance(&request, fixture.relational_runtime())
        .unwrap();
    assert!(Arc::ptr_eq(
        &task.current.as_ref().unwrap().original,
        &result
    ));
    let cursor = task.after.clone();
    assert!(matches!(
        task.advance(
            &ExecutionRequest::serial(&stopped),
            fixture.relational_runtime()
        ),
        Err(RuntimeWorldResultSnapshotCleanupDenial::Stopped(_))
    ));
    assert!(task.current.as_ref().unwrap().observer.is_some());
    assert_eq!(task.after, cursor);
    let foreign = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .build()
        .unwrap();
    assert!(matches!(
        task.advance(&request, &foreign),
        Err(RuntimeWorldResultSnapshotCleanupDenial::Native(
            NativeDenial::OwnerBindingMismatch
        ))
    ));
    assert!(task.current.as_ref().unwrap().observer.is_some());
    assert_eq!(task.after, cursor);
    let report = drive(&mut task, &request, fixture.relational_runtime());
    assert_eq!((report.released(), report.already_released()), (1, 0));
    let report = drive(
        &mut task,
        &ExecutionRequest::serial(&stopped),
        fixture.relational_runtime(),
    );
    assert_eq!((report.released(), report.already_released()), (1, 0));
    let (identity, basis, _) = facts.settled_relational_parts().unwrap();
    let invalid =
        facts.with_relational_settled_receipt(identity, basis, result.outcome().commit.clone());
    assert!(
        invalid.result_for_snapshot_cleanup().is_err(),
        "receipt-only Published is not live snapshot absence"
    );
}

#[test]
fn result_cleanup_lost_observer_rejoins_same_cell_and_observes_complete_before_late_stop() {
    let (fixture, owner, result, _) = published();
    let policy = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let native = fixture.relational_runtime();
    let mut task = owner.result_snapshot_cleanup_after_close(&request).unwrap();
    task.advance(&request, native).unwrap();
    task.advance(&request, native).unwrap();
    // Actual removal is performed, but its returned observer is lost.
    let original = task.current.as_ref().unwrap().original.clone();
    let observer = task
        .native
        .begin_commit_result_snapshot_cleanup(original.clone());
    assert!(matches!(
        observer.advance(&request, native),
        NativeProgress::Pending {
            cause: NativePending::Advanced,
            ..
        }
    ));
    assert!(task.current.as_ref().unwrap().observer.is_none());
    assert!(Arc::ptr_eq(&original, &result));
    let mut observer = task.native.begin_commit_result_snapshot_cleanup(original);
    loop {
        match observer.advance(&request, native) {
            NativeProgress::Pending {
                task: same,
                cause: NativePending::Advanced,
            } => observer = same,
            NativeProgress::Complete { result: same, .. } => {
                assert!(Arc::ptr_eq(&same, &result));
                break;
            }
            other => panic!("actual same-cell retirement: {other:?}"),
        }
    }
    let cancel = CancellationSource::new();
    let stopped = crate::test_execution_policy::serial_request().with_cancellation(cancel.token());
    cancel.cancel();
    let report = match task
        .advance(&ExecutionRequest::serial(&stopped), native)
        .unwrap()
    {
        RuntimeWorldResultSnapshotCleanupProgress::Complete(report) => report,
        other => panic!("last captured result must terminalize without another check: {other:?}"),
    };
    assert_eq!((report.released(), report.already_released()), (0, 1));
    assert!(task.current.is_none());
    assert_eq!(
        (
            drive(&mut task, &request, native).released(),
            task.counts.already_released
        ),
        (0, 1)
    );
}
