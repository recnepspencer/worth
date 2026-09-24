use super::*;
use worth_foundational::FoundationalBranchTarget;

#[test]
fn checkpoint_rejects_previous_branch_root_format_before_readmission() {
    let mut plan = checkpoint_recovery_plan("previous-branch-root-format");
    let root = selected_root_image(&mut plan);
    assert_eq!(root.format_version, 4);
    root.format_version = 3;
    recompute_partition_image_digest(root);
    assert_corrupt_recovery_leaves_no_recovered_state(
        plan,
        "unsupported branch-root image version `3`",
    );
}

#[test]
fn checkpoint_rejects_altered_root_content_with_recomputed_image_digest() {
    let mut plan = checkpoint_recovery_plan("altered-root-content");
    let root = selected_root_image(&mut plan);
    let generation = root.partition_images[0]
        .entity_arena
        .generations
        .first_mut()
        .expect("committed entity is present in its branch-root image");
    *generation = generation.wrapping_add(1);
    recompute_partition_image_digest(root);

    assert_corrupt_recovery_leaves_no_recovered_state(plan, "StorageContentMismatch");
}

#[test]
fn checkpoint_rejects_omitted_root_partition_with_recomputed_image_digest() {
    let mut plan = checkpoint_recovery_plan("omitted-root-partition");
    let root = selected_root_image(&mut plan);
    root.partition_images
        .pop()
        .expect("production checkpoint carries a branch-root partition");
    recompute_partition_image_digest(root);

    assert_corrupt_recovery_leaves_no_recovered_state(plan, "StorageContentMismatch");
}

#[test]
fn checkpoint_rejects_same_commit_truth_root_relabeling() {
    let mut plan = checkpoint_recovery_plan("checkpoint-truth-root-relabeling");
    let checkpoint = selected_main_checkpoint(&mut plan);
    let target = basis_target(checkpoint);
    let mut substituted_truth_root = *target.roots().truth_root();
    substituted_truth_root[0] ^= 0x80;
    replace_descriptor(
        checkpoint,
        crate::branch::RelationalBranchRootDescriptor::new(
            substituted_truth_root,
            *target.roots().schema_root(),
        ),
    );

    assert_corrupt_recovery_leaves_no_recovered_state(plan, "StorageContentMismatch");
}

#[test]
fn checkpoint_rejects_same_commit_schema_root_relabeling() {
    let mut plan = checkpoint_recovery_plan("checkpoint-schema-root-relabeling");
    let checkpoint = selected_main_checkpoint(&mut plan);
    let target = basis_target(checkpoint);
    let mut substituted_schema_root = *target.roots().schema_root();
    substituted_schema_root[0] ^= 0x40;
    replace_descriptor(
        checkpoint,
        crate::branch::RelationalBranchRootDescriptor::new(
            *target.roots().truth_root(),
            substituted_schema_root,
        ),
    );

    assert_corrupt_recovery_leaves_no_recovered_state(plan, "SchemaRootMismatch");
}

#[test]
fn tail_rejects_same_commit_root_descriptor_substitution() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "tail-descriptor-checkpoint");
    runtime
        .durability_authority()
        .checkpoint()
        .expect("production checkpoint succeeds");
    let tail_commit = create_entity_outcome(&runtime, "tail-descriptor-commit");
    let mut plan = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let checkpoint = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
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

    assert_corrupt_recovery_leaves_no_recovered_state(plan, "StorageContentMismatch");
}

#[test]
fn tail_replay_reconstructs_the_owner_content_root() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "tail-content-root-parity");
    let expected = current_main_descriptor(&runtime);
    let plan = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);

    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(plan)
        .expect("one production tail commit reconstructs");

    assert_eq!(current_main_descriptor(&recovered), expected);
}

#[test]
fn checkpoint_owner_root_is_stable_across_fingerprint_symbol_ids() {
    let plan = checkpoint_recovery_plan("fingerprint-symbol-stability");
    let mut left = plan.clone();
    let mut right = plan;
    install_fingerprint_image(&mut left, crate::symbols::data::Symbol(10_001));
    install_fingerprint_image(&mut right, crate::symbols::data::Symbol(20_001));

    let left_error = corrupt_recovery_error(left, "StorageContentMismatch");
    let right_error = corrupt_recovery_error(right, "StorageContentMismatch");

    assert_eq!(left_error.detail, right_error.detail);
}

fn checkpoint_recovery_plan(label: &str) -> RecoveryPlan {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, label);
    runtime
        .durability_authority()
        .checkpoint()
        .expect("production checkpoint succeeds");
    runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification)
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
    root.partition_image_digest =
        crate::durability::data::branch_root_partition_image_digest(&root.partition_images)
            .expect("mutated production image remains canonically encodable");
    root.root_image_digest = crate::durability::data::branch_root_image_digest(
        root.format_version,
        root.commit_id,
        root.partition_image_digest,
        root.schema_carrier_digest,
    );
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
        .push((symbol, "durability.fingerprint.family".to_owned()));
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

fn basis_target(
    checkpoint: &crate::branch::RelationalBranchCellCheckpoint,
) -> crate::branch::RelationalBranchTarget {
    match checkpoint.observation.target() {
        FoundationalBranchTarget::Basis(target) => target.clone(),
        FoundationalBranchTarget::Empty => panic!("committed branch cell has an exact target"),
    }
}

fn current_main_descriptor(
    runtime: &RelationalRuntime,
) -> crate::branch::RelationalBranchRootDescriptor {
    let state = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .expect("main branch exists");
    match state.observation().target() {
        FoundationalBranchTarget::Basis(target) => target.roots().clone(),
        FoundationalBranchTarget::Empty => panic!("committed main branch has an exact target"),
    }
}

fn replace_descriptor(
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

fn assert_corrupt_recovery_leaves_no_recovered_state(plan: RecoveryPlan, detail: &str) {
    let _ = corrupt_recovery_error(plan, detail);
}

fn corrupt_recovery_error(
    plan: RecoveryPlan,
    detail: &str,
) -> crate::durability::data::DurabilityError {
    let mut recovered = persisted_runtime_with_test_schema();
    let initial_cells = recovered.history.branch_cells_snapshot();
    assert!(initial_cells
        .iter()
        .all(|cell| matches!(cell.observation.target(), FoundationalBranchTarget::Empty)));
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("corrupt root binding cannot be readmitted");

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
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
        .branch_root_checkpoints()
        .expect("fresh branch cells contain no malformed roots")
        .is_empty());
    error
}
