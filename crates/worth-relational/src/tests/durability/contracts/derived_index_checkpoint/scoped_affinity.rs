//! Real checkpoint affinity: an inherited fork is legal; an unissued scope or missing payload is not.
use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};
use crate::identity::data::EntityId;
use crate::runtime::RelationalRuntime;

const NAME: &str = "scoped-checkpoint-shared-root";
const FORK: &str = "scoped-checkpoint-real-fork";

fn scoped_name_index(runtime: &RelationalRuntime) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "scoped-checkpoint-index".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity")
}

fn fork_checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    RecoveryPlan,
    EntityId,
    DerivedIndexId,
    crate::indexes::data::DerivedIndexGenerationId,
) {
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let anchor = create_entity_outcome(execution, &runtime, NAME);
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
        execution,
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
    medium.file_embedded_checkpoint(execution, &runtime);
    let plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    (plan, entity, index.index_id, generation)
}

fn name_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entity: EntityId,
) -> Option<String> {
    let identity = runtime.branch_identity(branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let read = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap()
        .read_entity(execution, entity)
        .unwrap();
    read.base().and_then(read_entity_name)
}

#[test]
fn real_shared_parent_root_scoped_checkpoint_restores_lookup_and_content() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (plan, entity, index, generation) = fork_checkpoint(execution);
    let restored_medium = OwnedTestDurableMedium::new();
    let mut restored = durable_runtime_for_medium(&restored_medium);
    restored
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
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
        .read_published_generation_for_observation(execution, index, &basis.observation())
        .unwrap()
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
        .execute_bounded_entity_field_lookup(
            request(),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap();
    assert_eq!(actual.generation_id(), generation);
    assert_eq!(actual.candidate_entity_ids(), &[entity]);
    let parity = restored
        .index_access()
        .execute_bounded_entity_field_lookup(
            request(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();
    assert_eq!(parity.candidate_entity_ids(), &[entity]);
    assert_eq!(
        name_on_branch(execution, &restored, &branch, entity),
        Some(NAME.into())
    );
    restored.snapshots().release_snapshot(&snapshot).unwrap();
}

#[test]
fn digest_valid_unissued_scoped_branch_denies_without_replacing_target_state() {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let (mut plan, _, _, _) = fork_checkpoint(execution);
        let checkpoint = plan.checkpoint.as_mut().unwrap();
        let existing = checkpoint.derived_index_checkpoint.as_ref().unwrap();
        let lane = fixture_lane;
        let mut generations = existing.readmit(&lane).unwrap();
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
                &lane,
                generations.into_iter().map(std::sync::Arc::new).collect(),
                existing.bases().to_vec(),
            )
            .unwrap();
        // Establish that the genuine delta/digest owner admits this wire payload:
        // the requested refusal must come from joining it to native branch authority.
        let decoded = artifact.readmit(&lane).unwrap();
        assert_eq!(decoded[0].source_branch_id, foreign);
        assert_eq!(decoded[0].applicability.branch_id, foreign);
        checkpoint.derived_index_checkpoint = Some(artifact);

        let target_medium = OwnedTestDurableMedium::new();
        let mut target = durable_runtime_for_medium(&target_medium);
        let retained = create_entity_outcome(execution, &target, "target-must-remain");
        let entity = changed_entities(&retained)[0];
        let index = scoped_name_index(&target);
        build(execution, &target, index.index_id, &retained);
        release_test_commit_snapshot(&target, &retained);
        let identity = target.main_branch_identity();
        let head = target.observe_branch(&identity).unwrap().0;
        let definitions = target.index_access().definitions_snapshot();
        let generations = target.index_access().generations_snapshot();
        assert_eq!(
            name_on_branch(execution, &target, &BranchId("main".into()), entity),
            Some("target-must-remain".into())
        );
        let error = target
            .durability_recovery()
            .recover(execution, plan)
            .expect_err("digest-valid cache metadata cannot invent a scoped branch");
        let error = crate::tests::support::expect_durability_failure(error);
        assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
        assert_eq!(target.observe_branch(&identity).unwrap().0, head);
        assert_eq!(target.index_access().definitions_snapshot(), definitions);
        assert_eq!(target.index_access().generations_snapshot(), generations);
        assert_eq!(
            name_on_branch(execution, &target, &BranchId("main".into()), entity),
            Some("target-must-remain".into())
        );
        assert!(target.branch_identity(&foreign).is_err());
    })
}

#[test]
fn checkpoint_index_artifact_missing_payload_denies_before_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "checkpoint-missing-index-payload");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.checkpoint.as_mut().unwrap().derived_index_checkpoint = None;
    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);

    let mut old_format = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    old_format
        .checkpoint
        .as_mut()
        .unwrap()
        .derived_index_checkpoint_format = 1;
    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, old_format)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
}

#[test]
fn checkpoint_recovery_denies_a_digest_valid_foreign_generation_affinity() {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_for_medium(&medium);
        let commit = create_entity_outcome(execution, &runtime, "foreign-affinity-index");
        let index = name_index(&runtime);
        build(execution, &runtime, index.index_id, &commit);
        release_test_commit_snapshot(&runtime, &commit);
        medium.file_embedded_checkpoint(execution, &runtime);
        let mut plan = medium
            .recovery_plan(
                execution,
                &runtime,
                RecoveryMode::NormalRecoveryVerification,
            )
            .unwrap();
        let checkpoint = plan.checkpoint.as_mut().unwrap();
        let existing = checkpoint.derived_index_checkpoint.as_ref().unwrap();
        let lane = fixture_lane;
        let mut generation = existing.readmit(&lane).unwrap().pop().unwrap();
        generation.applicability.branch_id = BranchId("foreign".into());
        checkpoint.derived_index_checkpoint = Some(
            crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::capture(
                &lane,
                vec![std::sync::Arc::new(generation)],
                existing.bases().to_vec(),
            )
            .unwrap(),
        );
        let recovered_medium = OwnedTestDurableMedium::new();
        let mut recovered = durable_runtime_for_medium(&recovered_medium);
        let error = recovered
            .durability_recovery()
            .recover(execution, plan)
            .unwrap_err();
        let error = crate::tests::support::expect_durability_failure(error);
        assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
        assert!(error.detail.contains("branch affinity"));
    })
}

pub(super) fn carried_index_producer(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::runtime::RelationalRuntime,
    crate::identity::data::EntityId,
    DerivedIndexId,
    DerivedIndexId,
    std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>,
    std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>,
) {
    let producer = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let register = |name: &str, aspect: &str| {
        producer
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: name.into(),
                branch_scoped: true,
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key(aspect), field_key(aspect)),
                },
            })
            .expect("index fixture has identity capacity")
            .index_id
    };
    let changed = register("complete-name", "name");
    let stable = register("complete-lifecycle", "lifecycle");
    let entity = create_entity(execution, &producer, "complete-index-value");
    let branch = BranchId("main".into());
    let built = producer.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: producer.history().branch_head(&branch).unwrap().commit_id,
            branch_id: branch.clone(),
            index_ids: vec![changed, stable],
        },
    );
    assert!(built.failed_indexes.is_empty());
    let latest = |index| {
        producer
            .index_access()
            .read_latest_generation(execution, index, &branch)
            .unwrap()
            .unwrap()
    };
    let carried = latest(stable);
    update_entity_and_release_snapshot(execution, &producer, entity, "complete-index-updated");
    let current = latest(changed);
    assert_eq!(latest(stable), carried);
    assert!(carried.applicability.version_id < current.applicability.version_id);
    (producer, entity, changed, stable, current, carried)
}
