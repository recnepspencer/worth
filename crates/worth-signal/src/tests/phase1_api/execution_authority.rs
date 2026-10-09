use super::source_corpus::{
    CHECKPOINT_IMAGE_SOURCE, HOT_PRECOMPUTE_SOURCE, HOT_SEMANTIC_FINALIZE_SOURCE,
    HOT_SERIAL_BATCH_SOURCE, HOT_STAGE_SOURCE, MERGE_EXECUTE_SOURCE,
    MERGE_RUNTIME_ARTIFACT_PROJECTION_SOURCE, MERGE_RUNTIME_EXECUTION_APPLICATION_SOURCE,
    PATCH_BUFFER_SOURCE, PLANNER_MODEL_SOURCE, PROOF_SOURCE, RUNTIME_SNAPSHOTTING_SOURCE,
    SEMANTIC_SOURCE, SNAPSHOT_RESTORE_SOURCE, WORKSPACE_SOURCE,
};

#[test]
fn gate3_finalize_paths_use_compact_artifact_images_instead_of_broad_runtime_state_snapshots() {
    for (name, source) in [
        ("semantic_finalize", HOT_SEMANTIC_FINALIZE_SOURCE),
        ("serial_batch", HOT_SERIAL_BATCH_SOURCE),
        ("stage", HOT_STAGE_SOURCE),
    ] {
        assert!(
            !source.contains("RuntimeArtifactState"),
            "{name} should not depend on broad RuntimeArtifactState in the finalize/apply carrier path"
        );
        assert!(
            !source.contains("node_runtime_artifact_state("),
            "{name} should not read broad runtime artifact state on the narrowed finalize/apply path"
        );
        assert!(
            source.contains("RuntimeArtifactFinalizeImage")
                || source.contains("node_runtime_artifact_finalize_image("),
            "{name} should consume the compact finalize image explicitly"
        );
    }
}

#[test]
fn gate4_stage_snapshot_commit_path_keeps_classified_snapshot_proofs() {
    assert!(
        HOT_STAGE_SOURCE.contains("apply_classified_snapshot_batch_commit("),
        "stage-owned snapshot publication should commit the already-classified proof form instead of reclassifying a generic batch late"
    );
    assert!(
        !HOT_STAGE_SOURCE.contains("apply_snapshot_batch_commit(stage_scratch.pending_snapshots)"),
        "stage-owned snapshot publication should not collapse back to generic snapshot batches once classification has occurred"
    );
}

#[test]
fn snapshot_proof_entries_are_not_publicly_forgeable() {
    assert!(
        PROOF_SOURCE.contains("pub struct PendingStableShapeSnapshotCommit {\r\n    node: NodeId,\r\n    update: VersionOnlySnapshotUpdate,\r\n    delta: SnapshotDeltaRecord,")
            || PROOF_SOURCE.contains("pub struct PendingStableShapeSnapshotCommit {\n    node: NodeId,\n    update: VersionOnlySnapshotUpdate,\n    delta: SnapshotDeltaRecord,"),
        "stable-shape snapshot proof entries should keep their fields private"
    );
    assert!(
        PROOF_SOURCE.contains("pub struct PendingReplacementSnapshotCommit {\r\n    node: NodeId,\r\n    update: ReplacementSnapshotUpdate,\r\n    delta: SnapshotDeltaRecord,")
            || PROOF_SOURCE.contains("pub struct PendingReplacementSnapshotCommit {\n    node: NodeId,\n    update: ReplacementSnapshotUpdate,\n    delta: SnapshotDeltaRecord,"),
        "replacement snapshot proof entries should keep their fields private"
    );
}

#[test]
fn lowered_execution_and_semantic_packets_use_constructors_instead_of_open_field_assembly() {
    assert!(
        PLANNER_MODEL_SOURCE.contains("impl LoweredTaskExecution"),
        "lowered execution should be mediated through an implementation boundary instead of remaining a raw field bag"
    );
    assert!(
        !HOT_STAGE_SOURCE.contains("LoweredTaskExecution {"),
        "stage lowering should construct lowered execution through its constructor rather than open field assembly"
    );
    assert!(
        HOT_STAGE_SOURCE.contains("LoweredTaskExecution::new("),
        "stage lowering should explicitly establish the lowered execution carrier through its constructor"
    );
    assert!(
        !HOT_STAGE_SOURCE.contains("SemanticTaskUpdate {"),
        "grouped apply reduction should construct semantic updates through a constructor rather than open field assembly"
    );
    assert!(
        SEMANTIC_SOURCE.contains("impl SemanticTaskUpdate"),
        "semantic update packets should be mediated through an implementation boundary"
    );
    assert!(
        HOT_SERIAL_BATCH_SOURCE.contains("ReadySerialFinalizeBatch::new("),
        "serial finalize readiness should be established through a constructor after width and snapshot checks"
    );
    assert!(
        !HOT_SERIAL_BATCH_SOURCE.contains("Ok(ReadySerialFinalizeBatch {"),
        "serial finalize readiness should not fall back to open struct assembly after proof checks"
    );
    assert!(
        WORKSPACE_SOURCE.contains("impl ConcurrentWorkerInput"),
        "parallel worker packets should be mediated through a construction boundary"
    );
    assert!(
        WORKSPACE_SOURCE.contains("impl ConcurrentApplyGroupInput"),
        "parallel grouped-input packets should be mediated through a construction boundary"
    );
    assert!(
        WORKSPACE_SOURCE.contains("impl GroupLocalTaskCommit"),
        "group-local commit packets should be mediated through a construction boundary"
    );
    assert!(
        WORKSPACE_SOURCE.contains("impl StageScratch"),
        "stage scratch should be mediated through owned transitions rather than open field access"
    );
    assert!(
        HOT_STAGE_SOURCE.contains("ConcurrentWorkerInput::new("),
        "parallel stage lowering should construct worker packets through their constructor"
    );
    assert!(
        HOT_STAGE_SOURCE.contains("ConcurrentApplyGroupInput::new("),
        "parallel stage lowering should construct grouped-input packets through their constructor"
    );
    assert!(
        HOT_STAGE_SOURCE.contains("GroupLocalTaskCommit::new("),
        "group-local apply packets should construct task commits through their constructor"
    );
    assert!(
        HOT_STAGE_SOURCE.contains("StageScratch::new("),
        "stage scratch should be constructed through its constructor on the grouped-apply path"
    );
    assert!(
        PLANNER_MODEL_SOURCE.contains("impl LoweredTask"),
        "lowered task packets should be mediated through an implementation boundary"
    );
    assert!(
        PLANNER_MODEL_SOURCE.contains("fn execution(&self) -> &LoweredTaskExecution"),
        "lowered task execution should be accessed through an accessor rather than a crate-visible field"
    );
    assert!(
        !PLANNER_MODEL_SOURCE.contains("pub(crate) execution: LoweredTaskExecution"),
        "lowered task should not expose its execution carrier as a crate-visible field"
    );
    assert!(
        HOT_STAGE_SOURCE.contains("LoweredTask::new("),
        "stage lowering should construct lowered tasks through their constructor"
    );
    assert!(
        !HOT_STAGE_SOURCE.contains("Ok(LoweredTask {"),
        "stage lowering should not fall back to open lowered-task assembly"
    );
    assert!(
        HOT_STAGE_SOURCE.contains("LoweredStagePlan::new("),
        "lowered stage plans should be constructed through their constructor"
    );
    assert!(
        !HOT_STAGE_SOURCE.contains("Ok(LoweredStagePlan {"),
        "stage lowering should not fall back to open lowered-stage assembly"
    );
}

#[test]
fn gate5_rollback_and_merge_paths_use_checkpoint_node_images_as_authority_boundary() {
    assert!(
        PATCH_BUFFER_SOURCE.contains("original: CheckpointNodeImage"),
        "transaction rollback patches should retain canonical checkpoint node images instead of raw NodeEntry clones"
    );
    assert!(
        PATCH_BUFFER_SOURCE.contains("node_checkpoint_image("),
        "transaction rollback should capture authority through the explicit checkpoint-image graph accessor"
    );
    assert!(
        PATCH_BUFFER_SOURCE.contains("replace_entry_from_checkpoint_image("),
        "transaction rollback should restore touched nodes through the checkpoint-image boundary"
    );
    assert!(
        !PATCH_BUFFER_SOURCE.contains("original: NodeEntry"),
        "transaction rollback should not keep raw NodeEntry snapshots as its authoritative rollback packet"
    );
    assert!(
        MERGE_EXECUTE_SOURCE.contains("node_checkpoint_image("),
        "merge adoption should request authority through the explicit checkpoint-image graph accessor"
    );
    assert!(
        MERGE_EXECUTE_SOURCE.contains("create_node_from_checkpoint_image("),
        "merge adoption should materialize introduced nodes through the checkpoint-image boundary"
    );
    assert!(
        MERGE_EXECUTE_SOURCE.contains("replace_node_from_checkpoint_image("),
        "merge adoption should route existing-target checkpoint replacement through the temporal-aware replacement boundary"
    );
    assert!(
        !MERGE_EXECUTE_SOURCE.contains("NodeEntry::from_checkpoint_image("),
        "merge adoption should not bounce checkpoint authority back through broad NodeEntry reconstruction"
    );
    assert!(
        MERGE_EXECUTE_SOURCE.contains("entry_image.set_eval_config("),
        "merge adoption should carry evaluation contract through the checkpoint image packet itself"
    );
    assert!(
        !MERGE_EXECUTE_SOURCE.contains("get_entry_mut("),
        "merge adoption should not fall back to broad mutable entry mutation after checkpoint-image materialization"
    );
    assert!(
        MERGE_RUNTIME_EXECUTION_APPLICATION_SOURCE.contains("replace_node_from_checkpoint_image("),
        "branch merge reconciliation should route existing targets through the branch-aware temporal replacement boundary"
    );
    assert!(
        MERGE_RUNTIME_EXECUTION_APPLICATION_SOURCE.contains("node_checkpoint_image("),
        "branch merge reconciliation should request checkpoint authority through the explicit graph checkpoint-image accessor"
    );
    assert!(
        !MERGE_RUNTIME_EXECUTION_APPLICATION_SOURCE
            .contains(".replace_entry(target_node, replacement)"),
        "branch merge reconciliation should not fall back to direct whole-entry replacement"
    );
    assert!(
        !MERGE_RUNTIME_ARTIFACT_PROJECTION_SOURCE.contains("get_runtime_artifact_state()"),
        "branch merge planning should not read broad runtime artifact state when hot/warm lane projections are available"
    );
    assert!(
        MERGE_RUNTIME_ARTIFACT_PROJECTION_SOURCE.contains("node_runtime_artifact_hot(")
            && MERGE_RUNTIME_ARTIFACT_PROJECTION_SOURCE.contains("node_runtime_artifact_warm("),
        "branch merge planning should derive merge comparability from explicit hot and warm artifact lanes"
    );
}

#[test]
fn gate5_snapshot_restore_uses_classified_snapshot_commit_boundary() {
    assert!(
        SNAPSHOT_RESTORE_SOURCE.contains("checkpoint_image")
            && SNAPSHOT_RESTORE_SOURCE.contains("dependency_snapshot_batch")
            && SNAPSHOT_RESTORE_SOURCE.contains(".classify()"),
        "snapshot restore planning should retain the classified checkpoint-carried dependency snapshot rebuild batch rather than only a generic batch form"
    );
    assert!(
        SNAPSHOT_RESTORE_SOURCE.contains("apply_classified_snapshot_batch_commit("),
        "snapshot restore execution should rebuild dependency snapshot state through the classified snapshot commit boundary"
    );
    assert!(
        SNAPSHOT_RESTORE_SOURCE.contains("restore_plan.checkpoint_restore_batch().clone_inner()"),
        "snapshot restore rebuild should consume the already-classified restore-plan batch instead of reclassifying the checkpoint batch late"
    );
    assert!(
        !SNAPSHOT_RESTORE_SOURCE.contains("dependency_snapshot_batch\n                        .clone()\n                        .classify()")
            && !SNAPSHOT_RESTORE_SOURCE.contains("dependency_snapshot_batch\r\n                        .clone()\r\n                        .classify()"),
        "snapshot restore rebuild should not reclassify dependency snapshot batches during execution"
    );
    assert!(
        !SNAPSHOT_RESTORE_SOURCE.contains("apply_snapshot_batch_commit(\r\n                        snapshot.checkpoint_image.dependency_snapshot_batch.clone(),")
            && !SNAPSHOT_RESTORE_SOURCE.contains("apply_snapshot_batch_commit(\n                        snapshot.checkpoint_image.dependency_snapshot_batch.clone(),"),
        "snapshot restore execution should not fall back to the generic snapshot batch commit path"
    );
    assert!(
        RUNTIME_SNAPSHOTTING_SOURCE.contains("apply_classified_snapshot_batch_commit("),
        "runtime branch snapshot restore should rebuild dependency snapshot state through the classified snapshot commit boundary"
    );
    assert!(
        RUNTIME_SNAPSHOTTING_SOURCE.contains("restore_plan.checkpoint_restore_batch().clone_inner()"),
        "runtime branch snapshot restore should consume the already-classified restore-plan batch instead of reclassifying the checkpoint batch late"
    );
    assert!(
        !RUNTIME_SNAPSHOTTING_SOURCE.contains("dependency_snapshot_batch\n                        .clone()\n                        .classify()")
            && !RUNTIME_SNAPSHOTTING_SOURCE.contains("dependency_snapshot_batch\r\n                        .clone()\r\n                        .classify()"),
        "runtime branch snapshot restore should not reclassify dependency snapshot batches during execution"
    );
    assert!(
        !RUNTIME_SNAPSHOTTING_SOURCE.contains("apply_snapshot_batch_commit(\r\n                        snapshot.checkpoint_image.dependency_snapshot_batch.clone(),")
            && !RUNTIME_SNAPSHOTTING_SOURCE.contains("apply_snapshot_batch_commit(\n                        snapshot.checkpoint_image.dependency_snapshot_batch.clone(),"),
        "runtime branch snapshot restore should not fall back to the generic snapshot batch commit path"
    );
}

#[test]
fn checkpoint_authority_image_fields_are_sealed_behind_methods() {
    assert!(
        CHECKPOINT_IMAGE_SOURCE.contains("pub struct CheckpointNodeImage {\n    state: NodeState,")
            || CHECKPOINT_IMAGE_SOURCE
                .contains("pub struct CheckpointNodeImage {\r\n    state: NodeState,"),
        "checkpoint authority image should keep its storage fields private"
    );
    assert!(
        !CHECKPOINT_IMAGE_SOURCE.contains("pub state:"),
        "checkpoint authority image should not expose raw state fields"
    );
    assert!(
        !CHECKPOINT_IMAGE_SOURCE.contains("pub dependencies_id:")
            && !CHECKPOINT_IMAGE_SOURCE.contains("pub runtime_artifact_state:")
            && !CHECKPOINT_IMAGE_SOURCE.contains("pub retained_artifact:")
            && !CHECKPOINT_IMAGE_SOURCE.contains("pub causality:")
            && !CHECKPOINT_IMAGE_SOURCE.contains("pub eval_config:"),
        "checkpoint authority image should not expose forgeable public fields"
    );
    assert!(
        CHECKPOINT_IMAGE_SOURCE.contains("pub(crate) fn set_eval_config(")
            && CHECKPOINT_IMAGE_SOURCE.contains("pub(crate) fn set_runtime_artifact_state(")
            && CHECKPOINT_IMAGE_SOURCE
                .contains("pub(crate) fn clear_dependency_handles_for_adoption("),
        "checkpoint authority image mutation should be mediated through crate-scoped methods"
    );
}

#[test]
fn reported_stage_outcomes_never_read_the_physical_high_water_mark() {
    // A shared authority's high-water mark moves with other leases, so it is
    // evidence only; the resolved posture alone decides what a stage reports.
    for (name, source) in [
        ("semantic_finalize", HOT_SEMANTIC_FINALIZE_SOURCE),
        ("stage", HOT_STAGE_SOURCE),
        ("precompute", HOT_PRECOMPUTE_SOURCE),
    ] {
        assert!(
            !source.contains("active_workers_high_watermark"),
            "{name} should decide parallel outcomes from the resolved posture"
        );
        assert!(
            source.contains("execution_resolved_parallel("),
            "{name} should share the planner's one parallel predicate"
        );
    }
}
