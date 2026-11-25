use super::*;
use worth_foundational::FoundationalBranchTarget;

#[test]
fn checkpoint_rejects_previous_branch_root_format_before_readmission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = checkpoint_recovery_plan(execution, "previous-branch-root-format");
    let root = selected_root_image(&mut plan);
    assert_eq!(
        root.format_version,
        crate::durability::data::DurableBranchRootImage::CURRENT_FORMAT_VERSION
    );
    let previous = root.format_version - 1;
    root.format_version = previous;
    recompute_partition_image_digest(root);
    let error = corrupt_recovery_error_with_class(
        execution,
        plan,
        RecoveryFailureClass::UnsupportedBranchRootImageVersion {
            found: previous,
            supported: crate::durability::data::DurableBranchRootImage::CURRENT_FORMAT_VERSION,
        },
        &format!("unsupported branch-root image version `{previous}`"),
    );
    assert!(error.detail.contains(&format!("`{previous}`")));
}

#[test]
fn checkpoint_rejects_altered_root_content_with_recomputed_image_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = checkpoint_recovery_plan(execution, "altered-root-content");
    let root = selected_root_image(&mut plan);
    let generation = root.partition_images[0]
        .entity_arena
        .generations
        .first_mut()
        .expect("committed entity is present in its branch-root image");
    *generation = generation.wrapping_add(1);
    recompute_partition_image_digest(root);

    assert_corrupt_recovery_leaves_no_recovered_state(execution, plan, "StorageContentMismatch");
}

#[test]
fn checkpoint_rejects_omitted_root_partition_with_recomputed_image_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = checkpoint_recovery_plan(execution, "omitted-root-partition");
    let root = selected_root_image(&mut plan);
    root.partition_images
        .pop()
        .expect("production checkpoint carries a branch-root partition");
    recompute_partition_image_digest(root);

    assert_corrupt_recovery_leaves_no_recovered_state(
        execution,
        plan,
        "root kind reconstruction: Damaged",
    );
}

#[test]
fn checkpoint_rejects_same_commit_truth_root_relabeling() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = checkpoint_recovery_plan(execution, "checkpoint-truth-root-relabeling");
    let target = basis_target(selected_main_checkpoint(&mut plan));
    let mut substituted_truth_root = *target.roots().truth_root();
    substituted_truth_root[0] ^= 0x80;
    let forged = crate::branch::RelationalBranchRootDescriptor::new(
        substituted_truth_root,
        *target.roots().schema_root(),
    );
    let mut cell_only = plan.clone();
    replace_descriptor(selected_main_checkpoint(&mut cell_only), forged.clone());
    assert_corrupt_recovery_leaves_no_recovered_state(
        execution,
        cell_only,
        "target does not match immutable commit artifact",
    );

    // Even without a canonical directory, relabeling the checkpoint commit
    // descriptor alongside the branch cell still fails closed at root
    // readmission against the reconstructed truth root.
    let commit_id = crate::history::data::CommitId(target.selected_commit_id());
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint selected");
    checkpoint.canonical_directory = None;
    checkpoint
        .commit_descriptors
        .insert(commit_id, forged.clone());
    replace_descriptor(selected_main_checkpoint(&mut plan), forged);

    assert_corrupt_recovery_leaves_no_recovered_state(execution, plan, "StorageContentMismatch");
}

#[test]
fn checkpoint_rejects_same_commit_schema_root_relabeling() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = checkpoint_recovery_plan(execution, "checkpoint-schema-root-relabeling");
    let target = basis_target(selected_main_checkpoint(&mut plan));
    let mut substituted_schema_root = *target.roots().schema_root();
    substituted_schema_root[0] ^= 0x40;
    let forged = crate::branch::RelationalBranchRootDescriptor::new(
        *target.roots().truth_root(),
        substituted_schema_root,
    );
    let mut cell_only = plan.clone();
    replace_descriptor(selected_main_checkpoint(&mut cell_only), forged.clone());
    assert_corrupt_recovery_leaves_no_recovered_state(
        execution,
        cell_only,
        "target does not match immutable commit artifact",
    );

    // Even without a canonical directory, relabeling the checkpoint commit
    // descriptor's schema root alongside the branch cell fails closed during
    // canonical commit admission before root readmission begins.
    let commit_id = crate::history::data::CommitId(target.selected_commit_id());
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint selected");
    checkpoint.canonical_directory = None;
    checkpoint
        .commit_descriptors
        .insert(commit_id, forged.clone());
    replace_descriptor(selected_main_checkpoint(&mut plan), forged);

    assert_corrupt_recovery_leaves_no_recovered_state(
        execution,
        plan,
        "checkpoint canonical admission: RootLinkage",
    );
}

#[test]
fn tail_rejects_same_commit_root_descriptor_substitution() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "tail-descriptor-checkpoint");
    capture_and_file_checkpoint(execution, &runtime, &medium);
    let tail_commit = create_entity_outcome(execution, &runtime, "tail-descriptor-commit");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let checkpoint = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test(execution))
        .find(|envelope| envelope.commit.commit_id == tail_commit.commit.commit_id)
        .and_then(|envelope| envelope.branch_cell_checkpoint.as_mut())
        .expect("production tail carries its exact pre-commit branch cell");
    let target = basis_target(checkpoint);
    let mut substituted_truth_root = *target.roots().truth_root();
    substituted_truth_root[0] ^= 0x20;
    replace_descriptor(
        checkpoint,
        crate::branch::RelationalBranchRootDescriptor::new(
            substituted_truth_root,
            *target.roots().schema_root(),
        ),
    );

    assert_corrupt_recovery_leaves_no_recovered_state(execution, plan, "StorageContentMismatch");
}

#[test]
fn checkpoint_owner_root_is_stable_across_fingerprint_symbol_ids() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let plan = checkpoint_recovery_plan(execution, "fingerprint-symbol-stability");
    let mut left = plan.clone();
    let mut right = plan;
    install_fingerprint_image(&mut left, crate::symbols::data::Symbol(10_001));
    install_fingerprint_image(&mut right, crate::symbols::data::Symbol(20_001));

    let left_error = corrupt_recovery_error(execution, left, "StorageContentMismatch");
    let right_error = corrupt_recovery_error(execution, right, "StorageContentMismatch");

    assert_eq!(left_error.detail, right_error.detail);
}

fn checkpoint_recovery_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
) -> RecoveryPlan {
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, label);
    capture_and_file_checkpoint(execution, &runtime, &medium);
    medium
        .expanded_recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
}

fn selected_root_image(
    plan: &mut RecoveryPlan,
) -> &mut crate::durability::data::DurableBranchRootImage {
    plan.checkpoint
        .as_mut()
        .and_then(|checkpoint| checkpoint.branch_roots.first_mut())
        .expect("production recovery plan selects a branch-root image")
}

fn recompute_partition_image_digest(root: &mut crate::durability::data::DurableBranchRootImage) {
    crate::durability::authority::recompute_branch_root_digests_for_test(root);
}

fn install_fingerprint_image(plan: &mut RecoveryPlan, symbol: crate::symbols::data::Symbol) {
    let checkpoint = plan
        .checkpoint
        .as_mut()
        .expect("production recovery plan selects a checkpoint");
    assert!(!checkpoint
        .symbol_table
        .entries
        .iter()
        .any(|(existing, _)| *existing == symbol));
    checkpoint
        .symbol_table
        .entries
        .push((symbol, "durability.fingerprint.family".into()));
    let root = checkpoint
        .branch_roots
        .first_mut()
        .expect("production checkpoint carries a branch-root image");
    root.partition_images[0].entity_arena.extra[0].structural_fingerprint = Some(
        crate::identity::data::StructuralFingerprint::new(symbol, 0x5eed),
    );
    recompute_partition_image_digest(root);
}

fn selected_main_checkpoint(
    plan: &mut RecoveryPlan,
) -> &mut crate::branch::RelationalBranchCellCheckpoint {
    plan.checkpoint
        .as_mut()
        .and_then(|checkpoint| {
            checkpoint
                .branch_cells
                .iter_mut()
                .find(|cell| cell.branch_id.0 == "main")
        })
        .expect("production recovery plan selects the main branch cell")
}

pub(super) fn basis_target(
    checkpoint: &crate::branch::RelationalBranchCellCheckpoint,
) -> crate::branch::RelationalBranchTarget {
    match checkpoint.observation.target() {
        FoundationalBranchTarget::Basis(target) => target.clone(),
        FoundationalBranchTarget::Empty => panic!("committed branch cell has an exact target"),
    }
}

pub(super) fn replace_descriptor(
    checkpoint: &mut crate::branch::RelationalBranchCellCheckpoint,
    descriptor: crate::branch::RelationalBranchRootDescriptor,
) {
    let target = basis_target(checkpoint);
    checkpoint.observation = crate::branch::relational_branch_observation(
        checkpoint.runtime_instance_id,
        &checkpoint.branch_id.0,
        FoundationalBranchTarget::basis(crate::branch::RelationalBranchTarget::new(
            target.runtime_instance_id(),
            target.selected_commit_id(),
            target.version_id(),
            target.parent_commit_ids().to_vec(),
            descriptor,
        )),
        checkpoint.observation.generation(),
    )
    .expect("descriptive corruption remains structurally well formed");
}

pub(super) fn assert_corrupt_recovery_leaves_no_recovered_state(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: RecoveryPlan,
    detail: &str,
) {
    let _ = corrupt_recovery_error(execution, plan, detail);
}

fn corrupt_recovery_error(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: RecoveryPlan,
    detail: &str,
) -> crate::durability::data::DurabilityError {
    corrupt_recovery_error_with_class(
        execution,
        plan,
        RecoveryFailureClass::CorruptCheckpoint,
        detail,
    )
}

fn corrupt_recovery_error_with_class(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: RecoveryPlan,
    expected_class: RecoveryFailureClass,
    detail: &str,
) -> crate::durability::data::DurabilityError {
    let medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&medium);
    let initial_cells = recovered.history.branch_cells_snapshot();
    assert!(initial_cells
        .iter()
        .all(|cell| matches!(cell.observation.target(), FoundationalBranchTarget::Empty)));
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("corrupt root binding cannot be readmitted");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, expected_class);
    assert!(
        error.detail.contains(detail),
        "unexpected denial: {error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    let recovered_cells = recovered.history.branch_cells_snapshot();
    assert_eq!(recovered_cells, initial_cells);
    assert!(recovered_cells
        .iter()
        .all(|cell| matches!(cell.observation.target(), FoundationalBranchTarget::Empty)));
    assert!(recovered
        .history
        .current_root_storage_inventory()
        .iter()
        .all(|root| root.commit_id().is_none() && root.descriptor().is_none()));
    error
}
