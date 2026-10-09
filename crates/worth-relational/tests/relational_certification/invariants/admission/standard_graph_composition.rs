use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

use super::graph_composition_probe::registration;
use super::graph_selected_state_probe::{registration as selected_state_registration, RULE_ID};
use super::world::supply_chain::{
    compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants,
    snapshot_for_supply_chain_identity, CompiledSupplyChainProgram, EntityKind, SupplyChainScale,
    SupplyChainWorldDefinition,
};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial;
use worth_relational::facade::runtime::{
    InvariantCatalog, InvariantCostClass, InvariantExecutionPoint, InvariantExecutionResult,
    InvariantReportedRule, InvariantVerdict, RelationalRuntime,
};
use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent,
    RecordRef, WorkerIntentBatch,
};

const COMMIT_PROBE_ID: &str = "phase5.common.commit-boundary";
const GRAPH_PROBE_ID: &str = "phase5.common.graph-composition";

#[test]
fn standard_runtime_public_graph_admission_is_touched_and_not_ordinary_commit_work() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::standard())
        .expect("Standard Supply Chain definition is valid");
    let program = CompiledSupplyChainProgram::compile(definition)
        .expect("Standard Supply Chain program compiles");
    let commit_probe = registration(
        COMMIT_PROBE_ID,
        InvariantExecutionPoint::CommitBoundary,
        InvariantCostClass::Global,
    );
    let graph_probe = registration(
        GRAPH_PROBE_ID,
        InvariantExecutionPoint::GraphComposition,
        InvariantCostClass::Touched,
    );
    let graph_preparation_calls = graph_probe.preparation_calls.clone();
    let graph_evaluation_calls = graph_probe.evaluation_calls.clone();
    let world =
        compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants(
            program,
            20_000,
            InvariantCatalog::default(),
            vec![commit_probe.registration, graph_probe.registration],
        )
        .expect("the Standard production baseline compiles");

    assert_eq!(
        custom_execution(&world.commit_result, COMMIT_PROBE_ID)
            .metadata()
            .max_cost(),
        InvariantCostClass::Global
    );
    assert!(!contains_custom_rule(&world.commit_result, GRAPH_PROBE_ID));
    assert_eq!(graph_preparation_calls.load(Ordering::Relaxed), 0);
    assert_eq!(graph_evaluation_calls.load(Ordering::Relaxed), 0);

    let branch = BranchId("main".to_owned());
    let graph_execution = graph_execution(&world.runtime, &branch);
    assert_eq!(
        graph_execution.metadata().max_cost(),
        InvariantCostClass::Touched
    );
    assert!(contains_custom_rule_in_execution(
        &graph_execution,
        GRAPH_PROBE_ID
    ));
    assert_eq!(graph_preparation_calls.load(Ordering::Relaxed), 1);
    assert_eq!(graph_evaluation_calls.load(Ordering::Relaxed), 1);
}

#[test]
fn graph_planning_uses_child_basis_after_main_diverges_and_rejects_stale_binding() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let program = CompiledSupplyChainProgram::compile(definition)
        .expect("Court Supply Chain program compiles");
    let selected_state_probe = selected_state_registration();
    let selected_state_expectation = selected_state_probe.expectation.clone();
    let world =
        compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants(
            program,
            20_000,
            InvariantCatalog::default(),
            vec![selected_state_probe.registration],
        )
        .expect("the Court production baseline compiles with selected-state evidence");
    let main = BranchId("main".to_owned());
    let (_, source) = world.runtime.observe_fork_source(&main).unwrap();
    let child = BranchId("graph-child".to_owned());
    world.runtime.fork_branch(child.clone(), source).unwrap();
    let child_identity = world.runtime.branch_identity(&child).unwrap();
    let child_version =
        snapshot_for_supply_chain_identity(&world.runtime, &child_identity).version_id();

    let main_only_entity = commit_graph_entity(&world.runtime, &main, "main-divergence");
    selected_state_expectation.forbid(main_only_entity);
    let main_identity = world.runtime.main_branch_identity();
    let main_version =
        snapshot_for_supply_chain_identity(&world.runtime, &main_identity).version_id();
    let main_execution = graph_execution(&world.runtime, &main);
    let child_execution = graph_execution(&world.runtime, &child);
    assert_ne!(main_version, child_version);
    assert!(matches!(
        custom_rule_verdict(&main_execution, RULE_ID),
        InvariantVerdict::Violation(_)
    ));
    assert!(matches!(
        custom_rule_verdict(&child_execution, RULE_ID),
        InvariantVerdict::Pass
    ));
    assert_eq!(child_execution.metadata().version_id(), child_version);
    assert_eq!(
        child_execution.metadata().current_version_id(),
        child_version
    );

    assert_graph_binding_stales_after_child_diverges(&world.runtime, &child);
}

fn assert_graph_binding_stales_after_child_diverges(runtime: &RelationalRuntime, child: &BranchId) {
    let child_identity = runtime.branch_identity(child).unwrap();
    let stale_options = runtime.admit_branch_basis(&child_identity).unwrap();
    commit_graph_entity(runtime, child, "child-divergence");
    let denied = runtime
        .begin_branch_transaction(
            &stale_options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap_err();
    assert_eq!(
        denied,
        RelationalBranchTransactionAdmissionDenial::StaleBasis
    );
}

fn custom_execution<'a>(
    commit: &'a worth_relational::facade::transactions::CommitResult,
    id: &str,
) -> &'a InvariantExecutionResult {
    commit
        .invariant_executions()
        .iter()
        .find(|execution| {
            execution.results().iter().any(|result| {
                matches!(
                    &result.rule,
                    InvariantReportedRule::Custom(identity) if identity.rule_id.as_str() == id
                )
            })
        })
        .unwrap_or_else(|| panic!("missing custom execution {id}"))
}

fn contains_custom_rule(
    commit: &worth_relational::facade::transactions::CommitResult,
    id: &str,
) -> bool {
    commit.invariant_executions().iter().any(|execution| {
        execution.results().iter().any(|result| {
            matches!(
                &result.rule,
                InvariantReportedRule::Custom(identity) if identity.rule_id.as_str() == id
            )
        })
    })
}

fn contains_custom_rule_in_execution(execution: &InvariantExecutionResult, id: &str) -> bool {
    execution.results().iter().any(|result| {
        matches!(
            &result.rule,
            InvariantReportedRule::Custom(identity) if identity.rule_id.as_str() == id
        )
    })
}

fn custom_rule_verdict<'a>(
    execution: &'a InvariantExecutionResult,
    id: &str,
) -> &'a InvariantVerdict {
    &execution
        .results()
        .iter()
        .find(|result| {
            matches!(
                &result.rule,
                InvariantReportedRule::Custom(identity) if identity.rule_id.as_str() == id
            )
        })
        .unwrap_or_else(|| panic!("missing custom execution {id}"))
        .verdict
}

fn graph_execution(runtime: &RelationalRuntime, branch: &BranchId) -> InvariantExecutionResult {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    let options = runtime
        .admit_branch_basis(&identity)
        .expect("transaction authority is owner-issued");
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            graph_entity_batch("common-graph-plan"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .graph_composition_plan(runtime)
        .expect("the Standard graph plan is branch-bound and owner-prepared")
}

fn commit_graph_entity(
    runtime: &RelationalRuntime,
    branch: &BranchId,
    client_key: &str,
) -> EntityId {
    let identity = runtime.branch_identity(branch).unwrap();
    let options = runtime.admit_branch_basis(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            graph_entity_batch(client_key),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let commit = transaction
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("branch divergence commits");
    commit
        .changed_records
        .iter()
        .find_map(|record| match *record {
            RecordRef::Entity(entity) => Some(entity),
            RecordRef::Relation(_) => None,
        })
        .expect("graph divergence creates one entity")
}

fn graph_entity_batch(client_key: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new(client_key).push(MutationIntent::Create(CreateIntent::Entity(
        EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: super::world::supply_chain::entity_kind_id(EntityKind::Vessel),
            client_key: worth_relational::facade::symbols::ClientKey::raw(client_key),
            fields: vessel_fields(client_key),
        },
    )))
}

fn vessel_fields(call_sign: &str) -> AspectFieldPatch {
    let mut fields = BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            AspectKey::new("call_sign").expect("call-sign aspect"),
            FieldKey::new("call_sign").expect("call-sign field"),
        ),
        AspectValue::String(InternedString::Raw(call_sign.to_owned())),
    );
    fields.insert(
        planned_single_field_locator(
            AspectKey::new("class").expect("class aspect"),
            FieldKey::new("class").expect("class field"),
        ),
        AspectValue::String(InternedString::Raw("Feeder".to_owned())),
    );
    fields.insert(
        planned_single_field_locator(
            AspectKey::new("capacity").expect("capacity aspect"),
            FieldKey::new("capacity").expect("capacity field"),
        ),
        AspectValue::UInt64(9_999),
    );
    fields.insert(
        planned_single_field_locator(
            AspectKey::new("posture").expect("posture aspect"),
            FieldKey::new("posture").expect("posture field"),
        ),
        AspectValue::String(InternedString::Raw("Open".to_owned())),
    );
    AspectFieldPatch::new(fields)
}
