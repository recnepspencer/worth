//! Authentic deletion metadata survives a cold reopen without folding history.
use super::*;
#[test]
fn cold_retirement_metadata_carries_original_creation_and_deletion_in_head() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let created = create_entity_outcome(execution, &producer, "retirement-before");
    let entity = changed_entities(&created)[0];
    let creation = created.version_id;
    release_test_commit_snapshot(&producer, &created);
    update_entity(execution, &producer, entity, "retirement-updated");
    let deleted = delete_entity(execution, &producer, entity);
    let deletion = deleted.version_id;
    release_test_commit_snapshot(&producer, &deleted);
    let (runtime, source, directory, _medium) =
        capture_and_reopen(execution, producer, medium, entity);
    let history = directory
        .mirror
        .iter()
        .chain(directory.roots.iter().flat_map(|root| &root.regions))
        .map(|region| region.history)
        .collect::<std::collections::BTreeSet<_>>();
    source.failures.lock().unwrap().extend(
        history
            .iter()
            .map(|member| (ColdRead::Member(*member), Cause::MissingMember)),
    );
    let snapshot = snapshot_for_owner_branch(
        &runtime,
        &worth_relational::facade::history::BranchId("main".into()),
    );
    let view = runtime.read_truth().project_snapshot(&snapshot).unwrap();
    let retirement = view
        .read_entity_retirement(execution, entity)
        .unwrap()
        .expect("the real deleted entity has its exact metadata");
    assert_eq!(retirement.entity_id(), entity);
    assert_eq!(retirement.created_at_version(), creation);
    assert_eq!(retirement.deleted_at_version(), deletion);
    let metadata = runtime
        .read_truth()
        .pin_resident_entity_metadata(&snapshot, entity)
        .unwrap()
        .unwrap();
    assert_eq!(metadata.live_kind(), None);
    assert_eq!(metadata.retirement(), Some(retirement));
    assert!(!source
        .member_reads
        .lock()
        .unwrap()
        .iter()
        .any(|member| history.contains(member)));
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}
