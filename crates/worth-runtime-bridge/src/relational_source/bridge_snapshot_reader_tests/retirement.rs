//! Genuine Bridge leases, original requests, and independent Native close evidence.
use super::*;
use crate::facade::{
    RelationalBridgeBasisRetirementDenial as Denial,
    RelationalBridgeBasisRetirementProgress as Progress, RelationalBridgeObservationRetirement,
};
use crate::relational_source::relational_test_support::release_test_commit_snapshot;
use crate::relational_source::runtime_source::{
    assert_observation_busy_and_indexes, assert_observation_index_refusals,
    assert_wrapped_successor_survives_abandonment,
};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::runtime::RelationalRuntimeCloseDenial;

#[test]
fn bridge_lease_retirement_actual_busy_keeps_both_commit_indexes_consistent() {
    let host = crate::snapshot::test_serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "indexed-retirement");
    release_test_commit_snapshot(&runtime, &created);
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "retirement").unwrap();
    let (_, basis) = source
        .observe_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let first = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let second = source.retain_branch_basis_for_bridge(&basis).unwrap();
    drop(basis);
    let mut first = first.into_retirement();
    let mut second = second.into_retirement();
    assert_observation_busy_and_indexes(&mut first, &source, &request, 1);
    assert_observation_busy_and_indexes(&mut second, &source, &request, 0);
    finish(&mut first, &request, &source);
    finish(&mut second, &request, &source);
    let first = first.into_retired().unwrap();
    let second = second.into_retired().unwrap();
    drop(source);
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
    first.reclaim();
    second.reclaim();
}

#[test]
fn bridge_lease_retirement_foreign_source_refuses_before_and_after_partial_progress() {
    let host = crate::snapshot::test_serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "foreign-retirement");
    release_test_commit_snapshot(&runtime, &created);
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "same-role").unwrap();
    let foreign =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "same-role").unwrap();
    let (_, basis) = source
        .observe_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let head = source.bind_branch_head_basis_for_bridge(&basis).unwrap();
    let snapshot = head.snapshot_identity().clone();
    drop(basis);
    let mut task = head.into_retirement();
    assert_eq!(
        task.advance(&request, &foreign),
        Progress::Refused(Denial::ForeignSource)
    );
    assert!(source.open_snapshot(&snapshot, request).is_ok());
    assert_eq!(task.advance(&request, &source), Progress::Advanced);
    assert_eq!(
        task.advance(&request, &foreign),
        Progress::Refused(Denial::ForeignSource)
    );
    assert!(source.open_snapshot(&snapshot, request).is_ok());
    for _ in 0..16 {
        match task.advance(&request, &source) {
            Progress::Advanced => {}
            Progress::Complete => break,
            progress => panic!("unexpected head retirement progress: {progress:?}"),
        }
    }
    let retired = task.into_retired().unwrap();
    assert!(source.open_snapshot(&snapshot, request).is_err());
    let (_, basis) = source
        .observe_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let direct = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let direct_snapshot = direct.snapshot_identity().clone();
    drop(basis);
    let mut direct = direct.into_retirement();
    assert_eq!(
        direct.advance(&request, &foreign),
        Progress::Refused(Denial::ForeignSource)
    );
    assert!(source.open_snapshot(&direct_snapshot, request).is_ok());
    assert_eq!(direct.advance(&request, &source), Progress::Advanced);
    assert!(source.open_snapshot(&direct_snapshot, request).is_err());
    let detached = retention(&runtime, &runtime.main_branch_identity());
    assert_eq!(
        direct.advance(&request, &foreign),
        Progress::Refused(Denial::ForeignSource)
    );
    assert_eq!(
        retention(&runtime, &runtime.main_branch_identity()),
        detached
    );
    finish(&mut direct, &request, &source);
    let direct = direct.into_retired().unwrap();
    drop(source);
    drop(foreign);
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
    retired.reclaim();
    direct.reclaim();
}

#[test]
fn bridge_lease_retirement_missing_or_inconsistent_original_index_refuses_owned() {
    let host = crate::snapshot::test_serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "negative-index-retirement");
    release_test_commit_snapshot(&runtime, &created);
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "retirement").unwrap();
    let (_, basis) = source
        .observe_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    drop(basis);
    let mut task = lease.into_retirement();
    assert_observation_index_refusals(&mut task, &source, &request);
    finish(&mut task, &request, &source);
    task.into_retired().unwrap().reclaim();
}

#[test]
fn bridge_lease_retirement_stop_after_bridge_detach_preserves_native_suffix() {
    let stop = CancellationSource::new();
    let host = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "partial-retirement");
    release_test_commit_snapshot(&runtime, &created);
    let branch = runtime.main_branch_identity();
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "retirement").unwrap();
    let (_, basis) = source.observe_branch_basis(&branch).unwrap();
    let lease = source.bind_branch_head_basis_for_bridge(&basis).unwrap();
    let snapshot = lease.snapshot_identity().clone();
    drop(basis);
    let before = retention(&runtime, &branch);
    let mut task = lease.into_retirement();
    for _ in 0..16 {
        if source.open_snapshot(&snapshot, request).is_err() {
            break;
        }
        assert_eq!(task.advance(&request, &source), Progress::Advanced);
    }
    assert!(source.open_snapshot(&snapshot, request).is_err());
    assert_eq!(
        retention(&runtime, &branch).external_pin_releases,
        before.external_pin_releases
    );
    stop.cancel();
    assert!(matches!(
        task.advance(&request, &source),
        Progress::Stopped(_)
    ));
    let task = task.into_retired().unwrap_err();
    assert_eq!(
        retention(&runtime, &branch).external_pin_releases,
        before.external_pin_releases
    );
    drop(task); // Explicitly unbudgeted abandonment releases the original suffix once.
    assert_eq!(
        retention(&runtime, &branch).external_pin_releases,
        before.external_pin_releases + 1
    );
    drop(source);
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
}

#[test]
fn bridge_lease_retirement_wrapped_head_successor_survives_abandonment() {
    let host = crate::snapshot::test_serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "wrapped-retirement");
    release_test_commit_snapshot(&runtime, &created);
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "same-role").unwrap();
    let foreign =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "same-role").unwrap();
    let (_, basis) = source
        .observe_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    assert_wrapped_successor_survives_abandonment(&source, &foreign, &basis, &request);
    drop(basis);
    drop(source);
    drop(foreign);
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
}

#[test]
fn bridge_lease_retirement_existing_reader_keeps_old_packet_while_new_opens_refuse() {
    let host = crate::snapshot::test_serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "retirement")
            .unwrap();
    let created = create_entity_outcome(&request, &runtime.lock().unwrap(), "alice");
    release_test_commit_snapshot(&runtime.lock().unwrap(), &created);
    let identity = runtime.lock().unwrap().main_branch_identity();
    let (_, basis) = source.observe_branch_basis(&identity).unwrap();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let snapshot = lease.snapshot_identity().clone();
    let reader = source.open_snapshot(&snapshot, request).unwrap();
    replace_entity_after_snapshot(&request, &mut runtime.lock().unwrap(), &created);
    drop(basis);
    let mut task = lease.into_retirement();
    finish(&mut task, &request, &source);
    let retired = task.into_retired().unwrap();
    assert!(source.open_snapshot(&snapshot, request).is_err());
    let packet = crate::facade::SnapshotReadPacket::new(vec![
        crate::facade::SnapshotReadRequest::for_relational_record(
            active_entity_identity(&created),
            SnapshotReadContract::scalar(aspect_key("name"), ScalarAspectType::String),
        ),
    ]);
    let result = reader.read_packet(&packet, request).unwrap();
    assert_eq!(result.snapshot_identity(), &snapshot);
    assert_eq!(
        result.records()[0].scalar_aspect_value(),
        Some(&AspectValue::String("alice".into()))
    );
    drop(result);
    drop(reader); // Reader/runtime-handle destruction is outside controlled retirement.
    retired.reclaim();
}

#[test]
fn bridge_lease_retirement_observation_alias_vetoes_unwrapped_native_owner_close() {
    let host = crate::snapshot::test_serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "independent-reader");
    release_test_commit_snapshot(&runtime, &created);
    let entity = changed_entities(&created)[0];
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "retirement").unwrap();
    let (_, basis) = source
        .observe_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let alias = basis.observation();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    drop(basis);
    let mut task = lease.into_retirement();
    finish(&mut task, &request, &source);
    assert_eq!(task.advance(&request, &source), Progress::Complete);
    let retired = task.into_retired().unwrap();
    drop(source);
    let runtime = Arc::try_unwrap(runtime).expect("the actual runtime outer Arc is unique");
    let refused = runtime.prepare_close().unwrap_err();
    assert!(matches!(
        refused.reason(),
        RelationalRuntimeCloseDenial::ReadCustodyRetained(_)
    ));
    let runtime = refused.into_runtime();
    assert!(runtime
        .entity_record_at_observation(&request, &alias, entity)
        .unwrap()
        .is_some());
    drop(alias);
    runtime.prepare_close().unwrap().finish();
    retired.reclaim();
}

fn finish(
    task: &mut RelationalBridgeObservationRetirement,
    request: &ExecutionRequest<'_, '_>,
    source: &RuntimeBridgeRelationalSource,
) {
    for _ in 0..16 {
        match task.advance(request, source) {
            Progress::Advanced => {}
            Progress::Complete => return,
            progress => panic!("unexpected observation retirement progress: {progress:?}"),
        }
    }
    panic!("observation retirement did not complete");
}
