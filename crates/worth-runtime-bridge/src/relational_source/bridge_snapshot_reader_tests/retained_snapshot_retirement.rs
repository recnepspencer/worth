//! Closed retained readers, live original owners and structural abandonment.
use super::*;
use crate::facade::{
    RelationalBridgeRetainedSnapshotRetirement,
    RelationalBridgeRetainedSnapshotRetirementDenial as Denial,
    RelationalBridgeRetainedSnapshotRetirementProgress as Progress, TruthSnapshotReader,
};
use crate::relational_source::relational_test_support::release_test_commit_snapshot;
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::change_source::RelationalRuntimeHandleAliasRetirementDenial as AliasDenial;

#[test]
fn source_alias_retirement_preserves_foreign_refusal_and_independent_read_custody() {
    let host = crate::snapshot::test_serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "retired-source");
    release_test_commit_snapshot(&runtime, &created);
    let source = RuntimeBridgeRelationalSource::for_graph_role(runtime.clone(), "primary").unwrap();
    let foreign = runtime_with_test_schema();
    let (source, denied) = match source.try_retire_alias_for_request(&request, &foreign) {
        Err(refused) => refused,
        Ok(_) => panic!("foreign owner must retain the original source"),
    };
    assert_eq!(denied, AliasDenial::ForeignOwner);
    let (_, basis) = source
        .observe_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let snapshot = source.open_retained_snapshot(lease).unwrap();
    let retired = source
        .try_retire_alias_for_request(&request, &runtime)
        .unwrap_or_else(|(_, cause)| panic!("actual source alias: {cause:?}"));
    assert_eq!(
        snapshot
            .read_packet(&packet(&created), request)
            .unwrap()
            .records()[0]
            .scalar_aspect_value(),
        Some(&AspectValue::String("retired-source".into()))
    );
    drop(snapshot);
    let runtime = Arc::try_unwrap(runtime).unwrap();
    let refused = runtime.prepare_close().unwrap_err();
    assert!(matches!(
        refused.reason(),
        worth_relational::facade::runtime::RelationalRuntimeCloseDenial::ReadCustodyRetained(_)
    ));
    drop(basis);
    refused.into_runtime().prepare_close().unwrap().finish();
    foreign.prepare_close().unwrap().finish();
    retired.reclaim();
}

#[test]
fn retained_snapshot_retirement_entry_refusal_returns_same_readable_foreign_and_shared_snapshot() {
    let host = crate::snapshot::test_serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "restored-reader");
    release_test_commit_snapshot(&runtime, &created);
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "same-role").unwrap();
    let foreign =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "same-role").unwrap();
    let (_, basis) = source
        .observe_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let original_identity = lease.snapshot_identity().clone();
    let snapshot = source.open_retained_snapshot(lease).unwrap();
    let (snapshot, denied) = snapshot
        .try_into_retirement(&foreign, &runtime)
        .unwrap_err();
    assert_eq!(
        denied,
        Denial::Binding(crate::facade::RelationalBridgeBasisRetirementDenial::ForeignSource)
    );
    assert_eq!(snapshot.snapshot_identity(), original_identity);
    assert_eq!(
        snapshot
            .read_packet(&packet(&created), request)
            .unwrap()
            .records()[0]
            .scalar_aspect_value(),
        Some(&AspectValue::String("restored-reader".into()))
    );
    let other = runtime_with_test_schema();
    let (snapshot, denied) = snapshot.try_into_retirement(&source, &other).unwrap_err();
    assert_eq!(denied, Denial::RuntimeAlias(AliasDenial::ForeignOwner));
    assert_eq!(snapshot.snapshot_identity(), original_identity);
    let mut task = snapshot.try_into_retirement(&source, &runtime).unwrap();
    assert_eq!(
        task.advance(&request, &foreign, &runtime),
        Progress::Refused(Denial::Binding(
            crate::facade::RelationalBridgeBasisRetirementDenial::ForeignSource
        ))
    );
    assert_eq!(
        task.advance(&request, &source, &runtime),
        Progress::Advanced
    );
    assert_eq!(
        task.advance(&request, &source, &other),
        Progress::Refused(Denial::RuntimeAlias(AliasDenial::ForeignOwner))
    );
    assert_eq!(
        task.advance(&request, &foreign, &runtime),
        Progress::Refused(Denial::Binding(
            crate::facade::RelationalBridgeBasisRetirementDenial::ForeignSource
        ))
    );
    finish(&mut task, &request, &source, &runtime);
    let retired = task.into_retired().unwrap();
    drop(basis);
    drop(source);
    drop(foreign);
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
    other.prepare_close().unwrap().finish();
    retired.reclaim();

    let shared = Arc::new(Mutex::new(runtime_with_test_schema()));
    let created = create_entity_outcome(&request, &shared.lock().unwrap(), "shared-reader");
    release_test_commit_snapshot(&shared.lock().unwrap(), &created);
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&shared), "shared")
            .unwrap();
    let identity = shared.lock().unwrap().main_branch_identity();
    let (_, basis) = source.observe_branch_basis(&identity).unwrap();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let snapshot = source.open_retained_snapshot(lease).unwrap();
    let (snapshot, denied) = snapshot
        .try_into_retirement(&source, &shared.lock().unwrap())
        .unwrap_err();
    assert_eq!(
        denied,
        Denial::RuntimeAlias(AliasDenial::SharedRepresentation)
    );
    assert_eq!(
        snapshot
            .read_packet(&packet(&created), request)
            .unwrap()
            .records()[0]
            .scalar_aspect_value(),
        Some(&AspectValue::String("shared-reader".into()))
    );
}

#[test]
fn retained_snapshot_retirement_reader_share_is_nonfinal_and_original_owner_still_commits() {
    let stop = CancellationSource::new();
    let host = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "anchored-reader");
    release_test_commit_snapshot(&runtime, &created);
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "retirement").unwrap();
    let branch = runtime.main_branch_identity();
    let (_, basis) = source.observe_branch_basis(&branch).unwrap();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let identity = lease.snapshot_identity().clone();
    let snapshot = source.open_retained_snapshot(lease).unwrap();
    drop(basis);
    let mut task = snapshot.try_into_retirement(&source, &runtime).unwrap();
    assert_eq!(
        task.advance(&request, &source, &runtime),
        Progress::Advanced
    );
    let later = create_entity_outcome(&request, &runtime, "genuine-commit-after-alias-discharge");
    release_test_commit_snapshot(&runtime, &later);
    assert!(later.commit.commit_id.0 > created.commit.commit_id.0);
    let before = retention(&runtime, &branch);
    assert_eq!(
        task.advance(&request, &source, &runtime),
        Progress::Advanced
    );
    assert_eq!(retention(&runtime, &branch), before);
    assert!(source.open_snapshot(&identity, request).is_ok());
    finish(&mut task, &request, &source, &runtime);
    assert!(source.open_snapshot(&identity, request).is_err());
    stop.cancel();
    assert_eq!(
        task.advance(&request, &source, &runtime),
        Progress::Complete
    );
    let retired = task.into_retired().unwrap();
    drop(source);
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
    retired.reclaim();
}

#[test]
fn retained_snapshot_retirement_stop_and_abandonment_preserve_each_original_suffix_once() {
    for completed_stages in 0..=4 {
        let stop = CancellationSource::new();
        let host = crate::policy::BridgeExecutionPolicyBaseline::operational()
            .serial_request(stop.token(), None);
        let request = ExecutionRequest::serial(&host);
        let runtime = Arc::new(runtime_with_test_schema());
        let created = create_entity_outcome(&request, &runtime, "stopped-reader");
        release_test_commit_snapshot(&runtime, &created);
        let source =
            RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "retirement")
                .unwrap();
        let branch = runtime.main_branch_identity();
        let (_, basis) = source.observe_branch_basis(&branch).unwrap();
        let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
        let snapshot = source.open_retained_snapshot(lease).unwrap();
        drop(basis);
        let before = retention(&runtime, &branch);
        let mut task = snapshot.try_into_retirement(&source, &runtime).unwrap();
        for _ in 0..completed_stages {
            assert_eq!(
                task.advance(&request, &source, &runtime),
                Progress::Advanced
            );
        }
        assert_eq!(retention(&runtime, &branch), before);
        stop.cancel();
        assert!(matches!(
            task.advance(&request, &source, &runtime),
            Progress::Stopped(_)
        ));
        let task = task.into_retired().unwrap_err();
        drop(task); // Explicitly unbudgeted; observation must precede raw lease.
        let after = retention(&runtime, &branch);
        assert_eq!(
            after.external_pin_releases,
            before.external_pin_releases + 1
        );
        assert_eq!(after.observation_releases, before.observation_releases + 1);
        drop(source);
        Arc::try_unwrap(runtime)
            .unwrap()
            .prepare_close()
            .unwrap()
            .finish();
    }
}

#[test]
fn retained_snapshot_retirement_independent_reader_survives_and_vetoes_owner_extraction() {
    let host = crate::snapshot::test_serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let created = create_entity_outcome(&request, &runtime, "independent-reader");
    release_test_commit_snapshot(&runtime, &created);
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "retirement").unwrap();
    let (_, basis) = source
        .observe_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let identity = lease.snapshot_identity().clone();
    let independent = source.open_snapshot(&identity, request).unwrap();
    let snapshot = source.open_retained_snapshot(lease).unwrap();
    drop(basis);
    let mut task = snapshot.try_into_retirement(&source, &runtime).unwrap();
    finish(&mut task, &request, &source, &runtime);
    assert!(source.open_snapshot(&identity, request).is_err());
    let retired = task.into_retired().unwrap();
    drop(source);
    let runtime =
        Arc::try_unwrap(runtime).expect_err("the independent reader owns its actual runtime alias");
    assert_eq!(
        independent
            .read_packet(&packet(&created), request)
            .unwrap()
            .records()[0]
            .scalar_aspect_value(),
        Some(&AspectValue::String("independent-reader".into()))
    );
    drop(independent); // Reader destruction remains outside controlled retirement.
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
    retired.reclaim();
}

fn packet(
    created: &worth_relational::facade::transactions::CommitResult,
) -> crate::facade::SnapshotReadPacket {
    crate::facade::SnapshotReadPacket::new(vec![
        crate::facade::SnapshotReadRequest::for_relational_record(
            active_entity_identity(created),
            SnapshotReadContract::scalar(aspect_key("name"), ScalarAspectType::String),
        ),
    ])
}

fn finish(
    task: &mut RelationalBridgeRetainedSnapshotRetirement,
    request: &ExecutionRequest<'_, '_>,
    source: &RuntimeBridgeRelationalSource,
    owner: &worth_relational::facade::runtime::RelationalRuntime,
) {
    for _ in 0..24 {
        match task.advance(request, source, owner) {
            Progress::Advanced => {}
            Progress::Complete => return,
            progress => panic!("unexpected retained-reader retirement progress: {progress:?}"),
        }
    }
    panic!("retained-reader retirement did not complete");
}
