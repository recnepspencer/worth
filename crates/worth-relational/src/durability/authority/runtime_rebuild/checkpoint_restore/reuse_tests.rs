//! Exact mirror reuse and global contract admission keep their existing courts.
use super::*;

#[test]
fn equal_exact_root_reuses_the_readmitted_partition_substrate() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = durable_runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &source, "reuse-root-mirror");
    release_test_commit_snapshot(&source, &committed);
    let checkpoint = crate::tests::support::checkpoint_durably(execution, &source).unwrap();
    assert_eq!(checkpoint.branch_roots.len(), 1);
    assert_eq!(
        checkpoint.branch_roots[0].partition_images,
        checkpoint.partition_images
    );

    let mut restored = durable_runtime_with_test_schema();
    let mut work = crate::durability::data::CheckpointRestoreWork::default();
    let roots =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            restore_branch_root_images(lane, execution, &mut restored, &checkpoint, &mut work, None)
        })
        .unwrap();
    let mirror =
        prepare_partitions(execution, &mut restored, &checkpoint, &roots, &mut work).unwrap();
    let root = roots.partitions.get(&committed.commit.commit_id).unwrap();
    assert!(shares_first_generation(&mirror, root));
}

#[test]
fn divergent_sibling_is_not_reused_as_the_storage_mirror() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = durable_runtime_with_test_schema();
    create_entity(execution, &source, "reuse-sibling-seed");
    let sibling = create_branch_from_main(&source, "reuse-sibling");
    let sibling_commit =
        create_entity_outcome_on_branch(execution, &source, "reuse-sibling-only", sibling.clone());
    release_test_commit_snapshot(&source, &sibling_commit);
    let main_commit = create_entity_outcome(execution, &source, "reuse-main-only");
    release_test_commit_snapshot(&source, &main_commit);
    let checkpoint = crate::tests::support::checkpoint_durably(execution, &source).unwrap();
    assert_eq!(checkpoint.branch_roots.len(), 2);

    let mut restored = durable_runtime_with_test_schema();
    let mut work = crate::durability::data::CheckpointRestoreWork::default();
    let roots =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            restore_branch_root_images(lane, execution, &mut restored, &checkpoint, &mut work, None)
        })
        .unwrap();
    let mirror =
        prepare_partitions(execution, &mut restored, &checkpoint, &roots, &mut work).unwrap();
    let main_root = roots.partitions.get(&main_commit.commit.commit_id).unwrap();
    let sibling_root = roots
        .partitions
        .get(&sibling_commit.commit.commit_id)
        .unwrap();
    assert!(shares_first_generation(&mirror, main_root));
    assert!(!shares_first_generation(&mirror, sibling_root));

    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &source,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.checkpoint = Some(checkpoint);
    let mut recovered = durable_runtime_with_test_schema();
    bind_durable_medium(&mut recovered, attached_test_medium(&source).attachment());
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for (branch, expected) in [(BranchId("main".into()), 2), (sibling, 2)] {
        let snapshot = snapshot_for_owner_branch(&recovered, &branch);
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot, execution)
                .expect("read request remains live")
                .unwrap()
                .entities()
                .len(),
            expected
        );
    }
}

#[test]
fn equal_image_cannot_bypass_global_contract_readmission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = durable_runtime_with_test_schema();
    create_entity(execution, &source, "reuse-contract-corruption");
    let mut checkpoint = crate::tests::support::checkpoint_durably(execution, &source).unwrap();
    assert_eq!(
        checkpoint.branch_roots[0].partition_images,
        checkpoint.partition_images
    );
    checkpoint.aspect_contracts.clear();

    let mut restored = durable_runtime_with_test_schema();
    let mut work = crate::durability::data::CheckpointRestoreWork::default();
    let roots =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            restore_branch_root_images(lane, execution, &mut restored, &checkpoint, &mut work, None)
        })
        .unwrap();
    let error =
        prepare_partitions(execution, &mut restored, &checkpoint, &roots, &mut work).unwrap_err();
    let RelationalRecoveryStop::Durability(error) = error else {
        panic!("contract corruption must remain a durability refusal: {error:?}");
    };
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("aspect readmission denied"));
}
