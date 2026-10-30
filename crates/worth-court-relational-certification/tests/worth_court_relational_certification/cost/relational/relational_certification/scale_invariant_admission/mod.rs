#[allow(unused_imports)]
use crate::relational_certification::growth_context::scale_invariant_admission::*;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::graph_composition_probe::registration};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::baseline_audit::audit as audit_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_uniqueness_assertion::assert_unique_conflict};
pub(crate) use {crate::relational_certification::growth_context::scale_invariant_admission::scale_snapshot_observation::current_snapshot_version, crate::relational_certification::growth_context::scale_invariant_admission::scale_snapshot_observation::live_record_count, crate::relational_certification::growth_context::scale_invariant_admission::scale_snapshot_observation::vessel_call_signs};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use worth_relational::facade::runtime::InvariantCatalog;

#[test]
#[ignore = "mandatory scheduled Scale certification; too expensive for the ordinary lane"]
fn large_runtime_keeps_global_enforcement_and_filters_graph_planning() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::scale())
        .expect("Scale Supply Chain definition is valid");
    let causal_record_count = definition.entities.len() + definition.relations.len();
    assert!(
        causal_record_count > 100_000,
        "the Scale fixture must exercise the real Large runtime path: {causal_record_count}"
    );
    let program = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(definition)
        .expect("Scale Supply Chain program compiles");
    let catalog = InvariantCatalog {
        registrations: vec![
            worth_relational::facade::runtime::InvariantRegistration::commit_boundary_blocking(
                worth_relational::facade::runtime::InvariantRule::unique_entity_aspect_field(
                    worth_foundational::facade::AspectKey::new("call_sign")
                        .expect("call-sign aspect"),
                    worth_foundational::facade::FieldKey::new("call_sign")
                        .expect("call-sign field"),
                ),
            ),
        ],
    };
    let commit_probe = registration(
        COMMIT_PROBE_ID,
        worth_relational::facade::runtime::InvariantExecutionPoint::CommitBoundary,
        worth_relational::facade::runtime::InvariantCostClass::Global,
    );
    let publication_probe = registration(
        PUBLICATION_PROBE_ID,
        worth_relational::facade::runtime::InvariantExecutionPoint::SnapshotPublication,
        worth_relational::facade::runtime::InvariantCostClass::Global,
    );
    let graph_probe = registration(
        GRAPH_PROBE_ID,
        worth_relational::facade::runtime::InvariantExecutionPoint::GraphComposition,
        worth_relational::facade::runtime::InvariantCostClass::Touched,
    );
    let graph_preparation_calls = graph_probe.preparation_calls.clone();
    let graph_evaluation_calls = graph_probe.evaluation_calls.clone();
    let custom = vec![
        commit_probe.registration,
        publication_probe.registration,
        graph_probe.registration,
    ];
    let world =
        compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants(
            execution, program, 200_000, catalog, custom,
        )
        .expect("the real Large causal baseline commits");
    let world = audit_supply_chain_baseline(execution, world)
        .expect("every installed Scale field, relation, schema axis, and ancestry row matches the independent oracle")
        .world;

    let commit_execution = custom_execution(&world.commit_result, COMMIT_PROBE_ID);
    assert_eq!(
        commit_execution.metadata().max_cost(),
        worth_relational::facade::runtime::InvariantCostClass::Global,
        "blocking commit enforcement retains its global correctness ceiling"
    );
    let baseline_publication_execution =
        custom_execution(&world.commit_result, PUBLICATION_PROBE_ID);
    assert_eq!(
        baseline_publication_execution.metadata().max_cost(),
        worth_relational::facade::runtime::InvariantCostClass::Global,
        "the first Large-sized publication still proves the global baseline"
    );
    assert!(!contains_custom_rule(&world.commit_result, GRAPH_PROBE_ID));
    assert!(contains_native_uniqueness(&world.commit_result));
    assert_eq!(
        graph_preparation_calls.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        graph_evaluation_calls.load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    assert_scale_fork_shares_complete_authority(execution, &world.runtime);

    let main = BranchId("main".to_owned());
    assert_eq!(
        live_record_count(execution, &world.runtime, &main),
        causal_record_count,
        "the real Large fixture must be installed in the selected production snapshot"
    );
    let graph_execution = graph_execution(execution, &world.runtime, &main);
    assert_eq!(
        graph_execution.metadata().max_cost(),
        worth_relational::facade::runtime::InvariantCostClass::Touched,
        "the direct GraphComposition profile retains its admitted Touched ceiling"
    );
    assert!(contains_custom_rule_in_execution(
        &graph_execution,
        GRAPH_PROBE_ID
    ));
    assert_eq!(
        graph_preparation_calls.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        graph_evaluation_calls.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    let graph_calls_before_follow_up = (
        graph_preparation_calls.load(std::sync::atomic::Ordering::Relaxed),
        graph_evaluation_calls.load(std::sync::atomic::Ordering::Relaxed),
    );
    let large_follow_up = commit_vessel(
        execution,
        &world.runtime,
        main.clone(),
        "large-follow-up-vessel",
        "FOLLOW-UP",
    )
    .expect("the post-baseline Large commit succeeds");
    let publication_execution = custom_execution(&large_follow_up, PUBLICATION_PROBE_ID);
    assert_eq!(
        publication_execution.metadata().execution_point(),
        worth_relational::facade::runtime::InvariantExecutionPoint::SnapshotPublication
    );
    assert_eq!(
        publication_execution.metadata().max_cost(),
        worth_relational::facade::runtime::InvariantCostClass::Partition,
        "the Large runtime lowers the ordinary publication planning ceiling"
    );
    assert_eq!(
        (
            graph_preparation_calls.load(std::sync::atomic::Ordering::Relaxed),
            graph_evaluation_calls.load(std::sync::atomic::Ordering::Relaxed),
        ),
        graph_calls_before_follow_up,
        "ordinary commit admission must not run GraphComposition probes"
    );
    let before_values = vessel_call_signs(execution, &world.runtime, &main);
    let before_branch = world
        .runtime
        .branch_reference_state(&main)
        .expect("main branch reference remains observable");
    let before_head = head_for_supply_chain_branch(&world.runtime, &main).version_id;
    let before_catalog = world.runtime.history().immutable_commit_count();
    let before_snapshot = current_snapshot_version(&world.runtime, &main);
    let duplicate = commit_vessel(
        execution,
        &world.runtime,
        main.clone(),
        "large-duplicate-vessel",
        DUPLICATE_CALL_SIGN,
    );
    assert_unique_conflict(duplicate.unwrap_err(), DUPLICATE_CALL_SIGN);
    assert_eq!(
        vessel_call_signs(execution, &world.runtime, &main),
        before_values
    );
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&main)
            .expect("rejected duplicate leaves main reference observable"),
        before_branch,
        "rejected Large duplicate must not leave a branch-reference residue"
    );
    assert_eq!(
        head_for_supply_chain_branch(&world.runtime, &main).version_id,
        before_head
    );
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        before_catalog
    );
    assert_eq!(
        current_snapshot_version(&world.runtime, &main),
        before_snapshot
    );
}
