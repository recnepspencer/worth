use super::*;
use crate::facade::indexes::DerivedIndexMaintenanceBudget;

fn name_index(runtime: &RelationalRuntime, name: &str, branch_scoped: bool) -> DerivedIndexId {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: name.into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped,
        })
        .expect("index fixture has identity capacity")
        .index_id
}

#[test]
fn a_fresh_fork_carries_authoring_generation_through_its_own_basis() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let anchor = create_entity_outcome(execution, &runtime, "fork-currency-anchor");
    let scoped = name_index(&runtime, "fork-currency.scoped", true);
    let global = name_index(&runtime, "fork-currency.global", false);
    let main = BranchId("main".into());
    let main_build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: main.clone(),
            index_ids: vec![scoped, global],
        },
    );
    assert!(main_build.failed_indexes.is_empty());
    let fork = create_branch_from_main(&runtime, "fork-currency");
    let (_, fork_basis) = runtime
        .observe_branch(&runtime.branch_identity(&fork).unwrap())
        .unwrap();
    let fork_observation = fork_basis.observation();
    let access = runtime.index_access();

    // Fork installation carries the exact source selection before exposing the target.
    assert_eq!(fork_observation.commit_id(), Some(anchor.commit.commit_id));
    let carried = access
        .read_published_generation_for_observation(execution, scoped, &fork_observation)
        .unwrap()
        .unwrap();
    assert_eq!(
        carried.generation_id,
        main_build.generations[0].generation_id
    );
    let shared = access
        .read_published_generation_for_observation(execution, global, &fork_observation)
        .unwrap()
        .unwrap();
    assert!(main_build
        .generations
        .iter()
        .any(|built| built.generation_id == shared.generation_id));

    // Reads reuse both carried selections without projecting the graph.
    let fork_build = runtime
        .index_authority()
        .refresh_for_basis(
            DerivedIndexBuildRequest {
                source_commit_id: anchor.commit.commit_id,
                branch_id: fork.clone(),
                index_ids: vec![scoped, global],
            },
            &fork_basis,
            None,
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(16),
                maximum_cold_record_slots: Some(0),
                maximum_derived_rows: Some(0),
            },
            execution,
        )
        .unwrap();
    assert_eq!(fork_build.generations.len(), 2);
    assert_eq!(fork_build.work.reused_generations, 2);
    assert_eq!(fork_build.work.seeded_generations, 0);
    assert_eq!(fork_build.work.cold_record_slots, 0);
    let fork_generation = access
        .read_published_generation_for_observation(execution, scoped, &fork_observation)
        .unwrap()
        .unwrap();
    assert!(fork_build
        .generations
        .iter()
        .any(|built| built.generation_id == fork_generation.generation_id));
    assert_eq!(fork_generation.source_branch_id, main);

    let (_, main_basis) = runtime
        .observe_branch(&runtime.branch_identity(&main).unwrap())
        .unwrap();
    let main_generation = access
        .read_published_generation_for_observation(execution, scoped, &main_basis.observation())
        .unwrap()
        .unwrap();
    assert_eq!(main_generation.source_branch_id, main);
    assert_eq!(main_generation.generation_id, fork_generation.generation_id);
    assert_eq!(main_generation.entries, fork_generation.entries);
}

#[test]
fn selected_fork_currency_uses_original_request_work_and_never_cold_population() {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
    use worth_execution::ExecutionWorkCeiling;

    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let anchor = create_entity_outcome(execution, &runtime, "selected-fork-currency");
    let index = name_index(&runtime, "selected-fork.name", true);
    let (_, main_basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(16),
        maximum_cold_record_slots: Some(0),
        maximum_derived_rows: Some(0),
    };
    let missing = runtime
        .index_authority()
        .ensure_field_index_for_basis(index, &main_basis, budget, execution)
        .unwrap_err();
    assert_eq!(missing.kind, Denial::ColdReconstructionRequired);
    let main_build = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index],
        },
        &main_basis,
    );
    assert!(main_build.failed_indexes.is_empty());
    // The production Query fork-name shape, rather than the short main key.
    let fork = create_branch_from_main(&runtime, "query-product-1-relational");
    let (_, basis) = runtime
        .observe_branch(&runtime.branch_identity(&fork).unwrap())
        .unwrap();
    let (short, _) = execution
        .run(ExecutionWorkCeiling::new(Some(0)), |_| {
            runtime
                .index_authority()
                .ensure_field_index_for_basis(index, &basis, budget, execution)
        })
        .unwrap();
    assert_eq!(
        short.unwrap_err().kind,
        Denial::Execution(Cause::WorkExhausted)
    );
    assert_eq!(
        runtime
            .index_access()
            .read_published_generation_for_observation(execution, index, &basis.observation(),)
            .unwrap()
            .unwrap()
            .generation_id,
        main_build.generations[0].generation_id
    );
    let (funded, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            runtime
                .index_authority()
                .ensure_field_index_for_basis(index, &basis, budget, execution)
        })
        .unwrap();
    funded.unwrap();
    assert!(report.charged_work() > 0);
    let fork_generation = runtime
        .index_access()
        .read_published_generation_for_observation(execution, index, &basis.observation())
        .unwrap()
        .unwrap();
    assert_eq!(
        fork_generation.generation_id,
        main_build.generations[0].generation_id
    );
    assert_eq!(fork_generation.entries, main_build.generations[0].entries);
    let (warm, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            runtime
                .index_authority()
                .ensure_field_index_for_basis(index, &basis, budget, execution)
        })
        .unwrap();
    warm.unwrap();
    assert!(report.charged_work() > 0);
    assert_eq!(
        runtime
            .index_access()
            .read_published_generation_for_observation(execution, index, &basis.observation())
            .unwrap()
            .unwrap()
            .generation_id,
        fork_generation.generation_id
    );
    release_test_commit_snapshot(&runtime, &anchor);
}

#[test]
fn fork_first_write_uses_carried_selection_after_parent_history_and_retirement() {
    use crate::transactions::data::{
        EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
    };
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let anchor = create_entity_outcome(execution, &runtime, "anchor");
    let entity = changed_entities(&anchor)[0];
    let index = name_index(&runtime, "fork-first-write.name", true);
    let untouched = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "fork-first-write.unreached".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::RelationField {
                field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            },
        })
        .unwrap()
        .index_id;
    let initial = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: anchor.commit.branch_id.clone(),
            index_ids: vec![index, untouched],
        },
    );
    assert!(initial.failed_indexes.is_empty());
    let retained = runtime
        .index_access()
        .read_published_generation_for_commit(execution, untouched, &anchor.commit)
        .unwrap()
        .unwrap();
    let fork = create_branch_from_main(&runtime, "fork-after-parent-advances");
    let mut previous = anchor;
    for step in 0..32 {
        let next = update_entity(execution, &runtime, entity, &format!("parent-{step}"));
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        runtime
            .index_authority()
            .refresh_for_basis(
                DerivedIndexBuildRequest {
                    source_commit_id: next.commit.commit_id,
                    branch_id: next.commit.branch_id.clone(),
                    index_ids: vec![index, untouched],
                },
                &basis,
                Some(&previous.snapshot),
                DerivedIndexMaintenanceBudget {
                    maximum_work_units: None,
                    maximum_cold_record_slots: Some(0),
                    maximum_derived_rows: None,
                },
                execution,
            )
            .unwrap();
        release_test_commit_snapshot(&runtime, &previous);
        previous = next;
    }
    runtime.retention().run_pass(execution).unwrap();
    let mut transaction = test_owner_begin_transaction_for_branch(&runtime, fork.clone());
    transaction
        .push_batch(
            WorkerIntentBatch::new("fork-first-write").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: name_field_patch("fork-value"),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let committed = super::candidate_index_publication::publish_native(
        execution,
        &runtime,
        runtime
            .prepare_branch_transaction(execution, transaction)
            .unwrap(),
    );
    let carried = runtime
        .index_access()
        .read_published_generation_for_commit(execution, untouched, &committed.commit)
        .unwrap()
        .unwrap();
    assert!(Arc::ptr_eq(&retained, &carried));
    let field = runtime
        .index_access()
        .read_published_generation_for_commit(execution, index, &committed.commit)
        .unwrap()
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.branch_identity(&fork).unwrap())
        .unwrap();
    let cold = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: committed.commit.commit_id,
            branch_id: fork,
            index_ids: vec![index],
        },
        &basis,
    );
    assert!(cold.failed_indexes.is_empty());
    assert_eq!(field.entries, cold.generations[0].entries);
    release_test_commit_snapshot(&runtime, &previous);
    release_test_commit_snapshot(&runtime, &committed);
}
