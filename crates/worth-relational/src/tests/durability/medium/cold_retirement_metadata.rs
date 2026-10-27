//! Authentic deletion metadata survives a cold reopen without folding history.
use super::*;
#[test]
fn cold_retirement_metadata_carries_original_creation_and_deletion_in_head() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let producer = durable_runtime_with_test_schema();
    let created = create_entity_outcome(execution, &producer, "retirement-before");
    let entity = changed_entities(&created)[0];
    let creation = created.version_id;
    release_test_commit_snapshot(&producer, &created);
    update_entity_and_release_snapshot(execution, &producer, entity, "retirement-updated");
    let deleted = delete_entity(execution, &producer, entity);
    let deletion = deleted.version_id;
    release_test_commit_snapshot(&producer, &deleted);
    let (recovery, source, _) = checkpoint_fixture(execution, producer, entity, false);
    source.damaged.store(false, Ordering::Relaxed);
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let manifest = rmp_serde::from_slice::<Input>(
        recovery
            .checkpoint()
            .unwrap()
            .checkpoint()
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap()
    .manifest;
    let history = manifest
        .mirror
        .iter()
        .chain(manifest.roots.iter().flat_map(|root| &root.regions))
        .map(|region| region.history)
        .collect::<std::collections::BTreeSet<_>>();
    source.failures.lock().unwrap().extend(
        history
            .iter()
            .map(|member| (ColdRead::Member(*member), Cause::MissingMember)),
    );
    let medium = TestDurableMedium::new().attachment();
    for bytes in source.commits.values() {
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            medium.publish(&artifact),
            crate::durability::medium::RelationalDurablePublication::Committed
        ));
    }
    let mut runtime = durable_runtime_with_test_schema();
    bind_durable_medium(&mut runtime, medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    let snapshot =
        snapshot_for_owner_branch(&runtime, &crate::history::data::BranchId("main".into()));
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
