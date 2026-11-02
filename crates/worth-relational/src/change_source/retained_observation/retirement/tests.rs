//! Issued custody, real owner gates, and authentic registration interleavings.
use super::*;
use crate::runtime::RelationalRuntimeCloseDenial;
use crate::tests::support::{create_entity, runtime_with_test_schema};
use std::sync::Arc;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn retained_observation_original_cut_and_lane_busy_retry_same_request() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    let mut task = retained.into_retirement();
    crate::history::retention::assert_external_cut_and_lane_busy(&mut task, &basis, &request);
    drop(basis);
    finish(&mut task, &request);
    let retired = task.into_retired().unwrap();
    runtime.prepare_close().unwrap().finish();
    retired.reclaim();
}

#[test]
fn retained_observation_stop_after_external_release_preserves_once_only_accounting() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let terminal = runtime
        .services
        .instrumentation
        .external_retention_terminal_accounting();
    let before = terminal.snapshot();
    let root = Arc::downgrade(&basis.inner.root);
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    drop(basis);
    let mut task = retained.into_retirement();
    assert_eq!(
        task.advance(&request),
        RelationalRetainedObservationRetirementProgress::Advanced
    );
    assert_eq!(
        terminal.snapshot().explicit_releases,
        before.explicit_releases + 1
    );
    stop.cancel();
    assert!(matches!(
        task.advance(&request),
        RelationalRetainedObservationRetirementProgress::Stopped(_)
    ));
    assert!(root.upgrade().is_some());
    assert_eq!(
        terminal.snapshot().dropped_releases,
        before.dropped_releases
    );
    let task = task.into_retired().unwrap_err();
    drop(task); // Explicitly unbudgeted abandonment joins remaining ordinary cleanup.
    assert_eq!(
        terminal.snapshot().explicit_releases,
        before.explicit_releases + 1
    );
    assert_eq!(
        terminal.snapshot().dropped_releases,
        before.dropped_releases
    );
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn retained_observation_registry_busy_retries_after_original_observation_release() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let (descriptor, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    drop(basis);
    let mut task = retained.into_retirement();
    crate::branch::assert_original_registry_busy(&mut task, &runtime, &descriptor, &request);
    task.into_retired().unwrap().reclaim();
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn retained_observation_original_prune_is_not_decremented_twice() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let (descriptor, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    drop(basis);
    let mut task = retained.into_retirement();
    crate::branch::assert_pruned_original_registration(&mut task, &runtime, &descriptor, &request);
    task.into_retired().unwrap().reclaim();
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn retained_observation_successor_registration_and_metrics_survive_original_retirement() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let (descriptor, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    drop(basis);
    let mut task = retained.into_retirement();
    crate::branch::assert_successor_registration_preserved(
        &mut task,
        &runtime,
        &descriptor,
        &request,
    );
    task.into_retired().unwrap().reclaim();
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn retained_observation_independent_alias_remains_readable_and_vetoes_close() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&request, &runtime, "original retained reader");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let alias = basis.observation();
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    drop(basis);
    let mut task = retained.into_retirement();
    finish(&mut task, &request);
    let retired = task.into_retired().unwrap();
    assert!(runtime
        .entity_record_at_observation(&request, &alias, entity)
        .unwrap()
        .is_some());
    let refused = runtime.prepare_close().unwrap_err();
    assert_eq!(
        refused.reason(),
        RelationalRuntimeCloseDenial::ReadCustodyRetained
    );
    let runtime = refused.into_runtime();
    assert!(runtime
        .entity_record_at_observation(&request, &alias, entity)
        .unwrap()
        .is_some());
    drop(alias);
    runtime.prepare_close().unwrap().finish();
    retired.reclaim();
}

#[test]
fn retained_observation_original_root_allocation_survives_retirement_until_reclaim() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let root = Arc::downgrade(&basis.inner.root);
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    drop(basis);
    let mut task = retained.into_retirement();
    finish(&mut task, &request);
    stop.cancel();
    assert_eq!(
        task.advance(&request),
        RelationalRetainedObservationRetirementProgress::Complete
    );
    let retired = task.into_retired().unwrap();
    runtime.prepare_close().unwrap().finish();
    assert!(
        root.upgrade().is_some(),
        "the original root allocation remains in retired custody"
    );
    retired.reclaim();
    assert!(
        root.upgrade().is_none(),
        "reclaim releases the same original root allocation"
    );
}

fn finish(task: &mut RelationalRetainedObservationRetirement, request: &ExecutionRequest<'_, '_>) {
    for _ in 0..12 {
        match task.advance(request) {
            RelationalRetainedObservationRetirementProgress::Advanced => {}
            RelationalRetainedObservationRetirementProgress::Complete => return,
            progress => panic!("unexpected retirement progress: {progress:?}"),
        }
    }
    panic!("original custody did not complete retirement");
}

#[test]
fn single_external_lease_retirement_preserves_an_independent_observation_veto() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    let accounting = runtime
        .services
        .instrumentation
        .external_retention_terminal_accounting();
    let before = accounting.snapshot();
    let mut task = retained.retention.into_observation_retirement();
    drop(retained.observation);
    finish(&mut task, &request);
    let retired = task.into_retired().unwrap();
    let refused = runtime.prepare_close().unwrap_err();
    assert_eq!(
        refused.reason(),
        RelationalRuntimeCloseDenial::ReadCustodyRetained
    );
    let runtime = refused.into_runtime();
    assert_eq!(
        accounting.snapshot().explicit_releases,
        before.explicit_releases + 1
    );
    assert_eq!(
        accounting.snapshot().dropped_releases,
        before.dropped_releases
    );
    drop(basis);
    runtime.prepare_close().unwrap().finish();
    retired.reclaim();
    assert_eq!(
        accounting.snapshot().explicit_releases,
        before.explicit_releases + 1
    );
    assert_eq!(
        accounting.snapshot().dropped_releases,
        before.dropped_releases
    );
}

#[test]
fn original_basis_retirement_preserves_independent_reads_and_retained_payload() {
    use crate::branch::{
        RelationalBranchBasisRetirementDenial as Denial,
        RelationalBranchBasisRetirementProgress as Progress,
    };
    let stop = CancellationSource::new();
    let live = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let stopped = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    stop.cancel();
    let request = ExecutionRequest::serial(&live);
    let runtime = runtime_with_test_schema();
    let foreign = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let independent = basis.clone();
    let root = Arc::downgrade(&basis.inner.root);
    let mut task = basis.into_retirement();
    assert!(matches!(
        task.advance(&request, &foreign),
        Err(Denial::ForeignOwner)
    ));
    assert!(matches!(
        task.advance(&ExecutionRequest::serial(&stopped), &runtime),
        Err(Denial::Execution(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(
        task.advance(&request, &runtime).unwrap(),
        Progress::Complete
    );
    let retired_share = task.into_retired().unwrap();
    let refused = runtime.prepare_close().unwrap_err();
    assert_eq!(
        refused.reason(),
        RelationalRuntimeCloseDenial::ReadCustodyRetained
    );
    let runtime = refused.into_runtime();
    assert_eq!(independent.identity(), &runtime.main_branch_identity());
    let mut last = independent.into_retirement();
    let mut complete = false;
    for _ in 0..8 {
        if last.advance(&request, &runtime).unwrap() == Progress::Complete {
            complete = true;
            break;
        }
    }
    assert!(
        complete,
        "the original Native basis must finish its actual lease retirement"
    );
    let retired_last = last.into_retired().unwrap();
    runtime.prepare_close().unwrap().finish();
    assert!(
        root.upgrade().is_some(),
        "the original root stays in retired payload custody"
    );
    drop(retired_share);
    drop(retired_last);
    assert!(
        root.upgrade().is_none(),
        "explicit reclaim destroys the same original allocation"
    );
    foreign.prepare_close().unwrap().finish();
}
