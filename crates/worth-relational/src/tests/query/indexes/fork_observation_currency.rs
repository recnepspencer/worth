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
        .index_id
}

#[test]
fn a_fresh_fork_needs_its_own_branch_scoped_generation() {
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

    // The fork selects its parent's commit, but a scoped generation published
    // for main is not current for the fork.
    assert_eq!(fork_observation.commit_id(), Some(anchor.commit.commit_id));
    assert!(access
        .published_generation_for_observation(scoped, &fork_observation)
        .is_none());
    let shared = access
        .published_generation_for_observation(global, &fork_observation)
        .unwrap();
    assert!(main_build
        .generations
        .iter()
        .any(|built| built.generation_id == shared.generation_id));

    // The production refresh reuses the shared global generation and seeds the
    // scoped one from main's generation at the same root. A zero cold budget
    // proves neither projects the graph.
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
                maximum_work_units: 16,
                maximum_cold_record_slots: 0,
                maximum_derived_rows: 0,
            },
            execution,
        )
        .unwrap();
    assert_eq!(fork_build.generations.len(), 2);
    assert_eq!(fork_build.work.reused_generations, 1);
    assert_eq!(fork_build.work.seeded_generations, 1);
    assert_eq!(fork_build.work.cold_record_slots, 0);
    let fork_generation = access
        .published_generation_for_observation(scoped, &fork_observation)
        .unwrap();
    assert!(fork_build
        .generations
        .iter()
        .any(|built| built.generation_id == fork_generation.generation_id));
    assert_eq!(fork_generation.source_branch_id, fork);

    let (_, main_basis) = runtime
        .observe_branch(&runtime.branch_identity(&main).unwrap())
        .unwrap();
    let main_generation = access
        .published_generation_for_observation(scoped, &main_basis.observation())
        .unwrap();
    assert_eq!(main_generation.source_branch_id, main);
    assert_ne!(main_generation.generation_id, fork_generation.generation_id);
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
        maximum_work_units: 16,
        maximum_cold_record_slots: 0,
        maximum_derived_rows: 0,
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
        .run(ExecutionWorkCeiling::new(0), |_| {
            runtime
                .index_authority()
                .ensure_field_index_for_basis(index, &basis, budget, execution)
        })
        .unwrap();
    assert_eq!(
        short.unwrap_err().kind,
        Denial::Execution(Cause::WorkExhausted)
    );
    assert!(runtime
        .index_access()
        .published_generation_for_observation(index, &basis.observation())
        .is_none());
    let (funded, report) = execution
        .run(ExecutionWorkCeiling::new(u64::MAX), |_| {
            runtime
                .index_authority()
                .ensure_field_index_for_basis(index, &basis, budget, execution)
        })
        .unwrap();
    funded.unwrap();
    assert!(report.charged_work() > 0);
    let fork_generation = runtime
        .index_access()
        .published_generation_for_observation(index, &basis.observation())
        .unwrap();
    assert_eq!(fork_generation.source_branch_id, fork);
    assert_eq!(fork_generation.entries, main_build.generations[0].entries);
    let (warm, report) = execution
        .run(ExecutionWorkCeiling::new(u64::MAX), |_| {
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
            .published_generation_for_observation(index, &basis.observation())
            .unwrap()
            .generation_id,
        fork_generation.generation_id
    );
    release_test_commit_snapshot(&runtime, &anchor);
}
