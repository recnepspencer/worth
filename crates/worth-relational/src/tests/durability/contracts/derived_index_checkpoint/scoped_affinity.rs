//! Real checkpoint affinity: an inherited fork is legal; an unissued scope is not.
use super::*;
use crate::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};
use crate::identity::data::EntityId;
use crate::runtime::RelationalRuntime;

const NAME: &str = "scoped-checkpoint-shared-root";
const FORK: &str = "scoped-checkpoint-real-fork";

fn fork_checkpoint() -> (
    RecoveryPlan,
    EntityId,
    DerivedIndexId,
    crate::indexes::data::DerivedIndexGenerationId,
) {
    let runtime = persisted_runtime_with_test_schema();
    let anchor = create_entity_outcome(&runtime, NAME);
    let entity = changed_entities(&anchor)[0];
    let branch = create_branch_from_main(&runtime, FORK);
    assert_ne!(anchor.commit.branch_id, branch);
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    assert_eq!(
        basis.observation().commit_id(),
        Some(anchor.commit.commit_id)
    );
    let index = scoped_name_index(&runtime);
    let built = runtime.index_authority().build_for_basis(
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch,
            index_ids: vec![index.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    assert_eq!(built.generations.len(), 1);
    let generation = built.generations[0].generation_id;
    drop(basis);
    release_test_commit_snapshot(&runtime, &anchor);
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    (plan, entity, index.index_id, generation)
}

fn name_on_branch(
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entity: EntityId,
) -> Option<String> {
    let identity = runtime.branch_identity(branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let read = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap()
        .read_entity(entity)
        .unwrap();
    read.base().and_then(read_entity_name)
}

#[test]
fn real_shared_parent_root_scoped_checkpoint_restores_lookup_and_content() {
    let (plan, entity, index, generation) = fork_checkpoint();
    let mut restored = persisted_runtime_with_test_schema();
    restored.durability_recovery().recover(plan).unwrap();
    let branch = BranchId(FORK.into());
    let identity = restored.branch_identity(&branch).unwrap();
    let (_, basis) = restored.observe_branch(&identity).unwrap();
    let artifact = restored
        .history
        .commit_artifact(basis.observation().commit_id().unwrap())
        .unwrap();
    assert_ne!(artifact.identity().authoring_branch(), &branch);
    let admitted = restored
        .index_access()
        .published_generation_for_observation(index, &basis.observation())
        .unwrap();
    assert_eq!(admitted.generation_id, generation);
    assert_eq!(admitted.source_branch_id, branch);
    let snapshot = snapshot_for_owner_branch(&restored, &branch);
    let request = || {
        BoundedEntityFieldLookupRequest::new(
            snapshot.clone(),
            index,
            KindId(1),
            aspect_field_locator(aspect_key("name"), field_key("name")),
            string_aspect_value(NAME),
            2,
        )
        .unwrap()
    };
    let actual = restored
        .index_access()
        .execute_bounded_entity_field_lookup(request(), BoundedIndexParityMode::Production)
        .unwrap();
    assert_eq!(actual.generation_id(), generation);
    assert_eq!(actual.candidate_entity_ids(), &[entity]);
    let parity = restored
        .index_access()
        .execute_bounded_entity_field_lookup(request(), BoundedIndexParityMode::Certification)
        .unwrap();
    assert_eq!(parity.candidate_entity_ids(), &[entity]);
    assert_eq!(
        name_on_branch(&restored, &branch, entity),
        Some(NAME.into())
    );
    restored.snapshots().release_snapshot(&snapshot).unwrap();
}

#[test]
fn digest_valid_unissued_scoped_branch_denies_without_replacing_target_state() {
    let (mut plan, _, _, _) = fork_checkpoint();
    let checkpoint = plan.checkpoint.as_mut().unwrap();
    let mut generations = checkpoint
        .derived_index_checkpoint
        .as_ref()
        .unwrap()
        .readmit()
        .unwrap();
    assert_eq!(generations.len(), 1);
    let foreign = BranchId("unissued-checkpoint-index-scope".into());
    assert!(checkpoint
        .branch_cells
        .iter()
        .all(|cell| cell.branch_id != foreign));
    generations[0].source_branch_id = foreign.clone();
    generations[0].applicability.branch_id = foreign.clone();
    let artifact =
        crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(
            generations.into_iter().map(std::sync::Arc::new).collect(),
        )
        .unwrap();
    // Establish that the genuine delta/digest owner admits this wire payload:
    // the requested refusal must come from joining it to native branch authority.
    let decoded = artifact.readmit().unwrap();
    assert_eq!(decoded[0].source_branch_id, foreign);
    assert_eq!(decoded[0].applicability.branch_id, foreign);
    checkpoint.derived_index_checkpoint = Some(artifact);

    let mut target = persisted_runtime_with_test_schema();
    let retained = create_entity_outcome(&target, "target-must-remain");
    let entity = changed_entities(&retained)[0];
    let index = scoped_name_index(&target);
    build(&target, index.index_id, &retained);
    release_test_commit_snapshot(&target, &retained);
    let identity = target.main_branch_identity();
    let head = target.observe_branch(&identity).unwrap().0;
    let definitions = target.index_access().definitions_snapshot();
    let generations = target.index_access().generations_snapshot();
    assert_eq!(
        name_on_branch(&target, &BranchId("main".into()), entity),
        Some("target-must-remain".into())
    );
    let error = target
        .durability_recovery()
        .recover(plan)
        .expect_err("digest-valid cache metadata cannot invent a scoped branch");
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert_eq!(target.observe_branch(&identity).unwrap().0, head);
    assert_eq!(target.index_access().definitions_snapshot(), definitions);
    assert_eq!(target.index_access().generations_snapshot(), generations);
    assert_eq!(
        name_on_branch(&target, &BranchId("main".into()), entity),
        Some("target-must-remain".into())
    );
    assert!(target.branch_identity(&foreign).is_err());
}
