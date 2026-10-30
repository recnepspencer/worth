#[allow(unused_imports)]
use crate::relational_certification::ordinary_context::standard_graph_composition::*;
pub(crate) use std::collections::BTreeMap;
pub(crate) use std::sync::atomic::Ordering;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::graph_composition_probe::registration};
pub(crate) use {crate::relational_certification::ordinary_context::graph_selected_state_probe::registration as selected_state_registration, crate::relational_certification::ordinary_context::graph_selected_state_probe::RULE_ID};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::PartitionId};
pub(crate) use {worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial};
pub(crate) use {worth_relational::facade::runtime::InvariantCatalog, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantExecutionResult, worth_relational::facade::runtime::InvariantReportedRule, worth_relational::facade::runtime::InvariantVerdict, worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RecordRef, worth_relational::facade::transactions::WorkerIntentBatch};

#[test]
fn standard_runtime_public_graph_admission_is_touched_and_not_ordinary_commit_work() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard())
        .expect("Standard Supply Chain definition is valid");
    let program = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(definition)
        .expect("Standard Supply Chain program compiles");
    let commit_probe = registration(
        COMMIT_PROBE_ID,
        worth_relational::facade::runtime::InvariantExecutionPoint::CommitBoundary,
        worth_relational::facade::runtime::InvariantCostClass::Global,
    );
    let graph_probe = registration(
        GRAPH_PROBE_ID,
        worth_relational::facade::runtime::InvariantExecutionPoint::GraphComposition,
        worth_relational::facade::runtime::InvariantCostClass::Touched,
    );
    let graph_preparation_calls = graph_probe.preparation_calls.clone();
    let graph_evaluation_calls = graph_probe.evaluation_calls.clone();
    let world =
        compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants(
            execution,
            program,
            20_000,
            worth_relational::facade::runtime::InvariantCatalog::default(),
            vec![commit_probe.registration, graph_probe.registration],
        )
        .expect("the Standard production baseline compiles");

    assert_eq!(
        custom_execution(&world.commit_result, COMMIT_PROBE_ID)
            .metadata()
            .max_cost(),
        worth_relational::facade::runtime::InvariantCostClass::Global
    );
    assert!(!contains_custom_rule(&world.commit_result, GRAPH_PROBE_ID));
    assert_eq!(
        graph_preparation_calls.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        graph_evaluation_calls.load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    let branch = BranchId("main".to_owned());
    let graph_execution = graph_execution(execution, &world.runtime, &branch);
    assert_eq!(
        graph_execution.metadata().max_cost(),
        worth_relational::facade::runtime::InvariantCostClass::Touched
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
}

#[test]
fn graph_planning_uses_child_basis_after_main_diverges_and_rejects_stale_binding() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let program = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(definition)
        .expect("Court Supply Chain program compiles");
    let selected_state_probe = selected_state_registration();
    let selected_state_expectation = selected_state_probe.expectation.clone();
    let world =
        compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants(
            execution,
            program,
            20_000,
            worth_relational::facade::runtime::InvariantCatalog::default(),
            vec![selected_state_probe.registration],
        )
        .expect("the Court production baseline compiles with selected-state evidence");
    let main = BranchId("main".to_owned());
    let (_, source) = world.runtime.observe_fork_source(&main).unwrap();
    let child = BranchId("graph-child".to_owned());
    world
        .runtime
        .fork_branch(execution, child.clone(), source)
        .unwrap();
    let child_identity = world.runtime.branch_identity(&child).unwrap();
    let child_version =
        snapshot_for_supply_chain_identity(&world.runtime, &child_identity).version_id();

    let main_only_entity = commit_graph_entity(execution, &world.runtime, &main, "main-divergence");
    selected_state_expectation.forbid(main_only_entity);
    let main_identity = world.runtime.main_branch_identity();
    let main_version =
        snapshot_for_supply_chain_identity(&world.runtime, &main_identity).version_id();
    let main_execution = graph_execution(execution, &world.runtime, &main);
    let child_execution = graph_execution(execution, &world.runtime, &child);
    assert_ne!(main_version, child_version);
    assert!(matches!(
        custom_rule_verdict(&main_execution, RULE_ID),
        worth_relational::facade::runtime::InvariantVerdict::Violation(_)
    ));
    assert!(matches!(
        custom_rule_verdict(&child_execution, RULE_ID),
        worth_relational::facade::runtime::InvariantVerdict::Pass
    ));
    assert_eq!(child_execution.metadata().version_id(), child_version);
    assert_eq!(
        child_execution.metadata().current_version_id(),
        child_version
    );

    assert_graph_binding_stales_after_child_diverges(execution, &world.runtime, &child);
}
