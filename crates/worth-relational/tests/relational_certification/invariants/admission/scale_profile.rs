use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

#[path = "scale_snapshot_observation.rs"]
mod scale_snapshot_observation;

use super::graph_composition_probe::registration;
use super::world::supply_chain::{
    audit_supply_chain_baseline,
    compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants,
    entity_kind_id, head_for_supply_chain_branch, CompiledSupplyChainProgram, EntityKind,
    SupplyChainScale, SupplyChainWorldDefinition,
};
use crate::invariant_uniqueness_assertion::assert_unique_conflict;
use scale_snapshot_observation::{current_snapshot_version, live_record_count, vessel_call_signs};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::runtime::{
    InvariantCatalog, InvariantCostClass, InvariantExecutionPoint, InvariantExecutionResult,
    InvariantRegistration, InvariantReportedRule, InvariantRule, RelationalRuntime,
};
use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent,
    TransactionCommitError, WorkerIntentBatch,
};

const COMMIT_PROBE_ID: &str = "phase5.large.commit-boundary";
const PUBLICATION_PROBE_ID: &str = "phase5.large.publication";
const GRAPH_PROBE_ID: &str = "phase5.large.graph-composition";
const DUPLICATE_CALL_SIGN: &str = "AURORA";

#[test]
#[ignore = "mandatory scheduled Scale certification; too expensive for the ordinary lane"]
fn large_runtime_keeps_global_enforcement_and_filters_graph_planning() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::scale())
        .expect("Scale Supply Chain definition is valid");
    let causal_record_count = definition.entities.len() + definition.relations.len();
    assert!(
        causal_record_count > 100_000,
        "the Scale fixture must exercise the real Large runtime path: {causal_record_count}"
    );
    let program = CompiledSupplyChainProgram::compile(definition)
        .expect("Scale Supply Chain program compiles");
    let catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(
                AspectKey::new("call_sign").expect("call-sign aspect"),
                FieldKey::new("call_sign").expect("call-sign field"),
            ),
        )],
    };
    let commit_probe = registration(
        COMMIT_PROBE_ID,
        InvariantExecutionPoint::CommitBoundary,
        InvariantCostClass::Global,
    );
    let publication_probe = registration(
        PUBLICATION_PROBE_ID,
        InvariantExecutionPoint::SnapshotPublication,
        InvariantCostClass::Global,
    );
    let graph_probe = registration(
        GRAPH_PROBE_ID,
        InvariantExecutionPoint::GraphComposition,
        InvariantCostClass::Touched,
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
            program, 200_000, catalog, custom,
        )
        .expect("the real Large causal baseline commits");
    let world = audit_supply_chain_baseline(world)
        .expect("every installed Scale field, relation, schema axis, and ancestry row matches the independent oracle")
        .world;

    let commit_execution = custom_execution(&world.commit_result, COMMIT_PROBE_ID);
    assert_eq!(
        commit_execution.metadata().max_cost(),
        InvariantCostClass::Global,
        "blocking commit enforcement retains its global correctness ceiling"
    );
    let baseline_publication_execution =
        custom_execution(&world.commit_result, PUBLICATION_PROBE_ID);
    assert_eq!(
        baseline_publication_execution.metadata().max_cost(),
        InvariantCostClass::Global,
        "the first Large-sized publication still proves the global baseline"
    );
    assert!(!contains_custom_rule(&world.commit_result, GRAPH_PROBE_ID));
    assert!(contains_native_uniqueness(&world.commit_result));
    assert_eq!(graph_preparation_calls.load(Ordering::Relaxed), 0);
    assert_eq!(graph_evaluation_calls.load(Ordering::Relaxed), 0);

    assert_scale_fork_shares_complete_authority(&world.runtime);

    let main = BranchId("main".to_owned());
    assert_eq!(
        live_record_count(&world.runtime, &main),
        causal_record_count,
        "the real Large fixture must be installed in the selected production snapshot"
    );
    let graph_execution = graph_execution(&world.runtime, &main);
    assert_eq!(
        graph_execution.metadata().max_cost(),
        InvariantCostClass::Touched,
        "the direct GraphComposition profile retains its admitted Touched ceiling"
    );
    assert!(contains_custom_rule_in_execution(
        &graph_execution,
        GRAPH_PROBE_ID
    ));
    assert_eq!(graph_preparation_calls.load(Ordering::Relaxed), 1);
    assert_eq!(graph_evaluation_calls.load(Ordering::Relaxed), 1);
    let graph_calls_before_follow_up = (
        graph_preparation_calls.load(Ordering::Relaxed),
        graph_evaluation_calls.load(Ordering::Relaxed),
    );
    let large_follow_up = commit_vessel(
        &world.runtime,
        main.clone(),
        "large-follow-up-vessel",
        "FOLLOW-UP",
    )
    .expect("the post-baseline Large commit succeeds");
    let publication_execution = custom_execution(&large_follow_up, PUBLICATION_PROBE_ID);
    assert_eq!(
        publication_execution.metadata().execution_point(),
        InvariantExecutionPoint::SnapshotPublication
    );
    assert_eq!(
        publication_execution.metadata().max_cost(),
        InvariantCostClass::Partition,
        "the Large runtime lowers the ordinary publication planning ceiling"
    );
    assert_eq!(
        (
            graph_preparation_calls.load(Ordering::Relaxed),
            graph_evaluation_calls.load(Ordering::Relaxed),
        ),
        graph_calls_before_follow_up,
        "ordinary commit admission must not run GraphComposition probes"
    );
    let before_values = vessel_call_signs(&world.runtime, &main);
    let before_branch = world
        .runtime
        .branch_reference_state(&main)
        .expect("main branch reference remains observable");
    let before_head = head_for_supply_chain_branch(&world.runtime, &main).version_id;
    let before_catalog = world.runtime.history().immutable_commit_count();
    let before_snapshot = current_snapshot_version(&world.runtime, &main);
    let duplicate = commit_vessel(
        &world.runtime,
        main.clone(),
        "large-duplicate-vessel",
        DUPLICATE_CALL_SIGN,
    );
    assert_unique_conflict(duplicate.unwrap_err(), DUPLICATE_CALL_SIGN);
    assert_eq!(vessel_call_signs(&world.runtime, &main), before_values);
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

fn assert_scale_fork_shares_complete_authority(runtime: &RelationalRuntime) {
    let main = BranchId("main".to_owned());
    let (_, source) = runtime
        .observe_fork_source(&main)
        .expect("Scale main remains an owner-observed fork source");
    let child = BranchId("scale-zero-copy".to_owned());
    runtime
        .fork_branch(child.clone(), source)
        .expect("Scale fork remains metadata-only");
    let identities = [
        runtime.main_branch_identity(),
        runtime.branch_identity(&child).unwrap(),
    ];
    let sharing = runtime.observe_branch_sharing(&identities).unwrap();

    assert_eq!(sharing.unique_root_count(), 1);
    assert_eq!(sharing.unique_canonical_commit_artifacts(), 1);
    assert_eq!(sharing.fork_materialized_entity_count(), 0);
    assert_eq!(sharing.fork_materialized_relation_count(), 0);
    assert_eq!(sharing.fork_materialized_authoritative_bytes(), 0);
    assert_eq!(sharing.copied_commit_envelopes(), 0);
    assert_eq!(
        sharing
            .coordination_cells()
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        2
    );
    assert!(
        sharing.logical_branch_authoritative_bytes()
            > sharing.unique_physical_authoritative_bytes(),
        "two Scale branches must expose two logical views of one physical authority graph"
    );
}

fn custom_execution<'a>(
    commit: &'a worth_relational::facade::transactions::CommitResult,
    id: &str,
) -> &'a worth_relational::facade::runtime::InvariantExecutionResult {
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

fn contains_native_uniqueness(
    commit: &worth_relational::facade::transactions::CommitResult,
) -> bool {
    commit.invariant_executions().iter().any(|execution| {
        execution.results().iter().any(|result| {
            matches!(
                &result.rule,
                InvariantReportedRule::Native(InvariantRule::UniqueEntityAspectField { .. })
            )
        })
    })
}

fn commit_vessel(
    runtime: &RelationalRuntime,
    branch: BranchId,
    client_key: &str,
    call_sign: &str,
) -> Result<worth_relational::facade::transactions::CommitResult, TransactionCommitError> {
    let identity = runtime
        .branch_identity(&branch)
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
            WorkerIntentBatch::new(client_key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: entity_kind_id(EntityKind::Vessel),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(client_key),
                    fields: vessel_fields(call_sign),
                },
            ))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
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
            WorkerIntentBatch::new("large-graph-plan").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: entity_kind_id(EntityKind::Vessel),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(
                        "large-graph-plan",
                    ),
                    fields: vessel_fields("GRAPH-PLAN"),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .graph_composition_plan(runtime)
        .expect("the real Scale graph plan is branch-bound and owner-prepared")
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
