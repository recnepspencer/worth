use std::collections::BTreeSet;

use super::invariant_oracle_expectations::expected_supply_chain_branch;
use super::world::supply_chain::{
    assert_oracle_matches, certified_supply_chain_world, commit_branch_batch, compare,
    lower_supply_chain_production_delta, observe_supply_chain_snapshot,
    snapshot_for_supply_chain_identity, BranchLabel, DeltaId, ObservedSupplyChainState,
    ProductionSeededSupplyChainWorld, SupplyChainScale,
};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::{
    RelationalBranchSharingCostCounters, RelationalMvccCostScope,
};
use worth_relational::facade::query::PlannedQueryPacket;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, EntityMutationIntent, MutationIntent,
    RecordRef, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

#[test]
fn main_advances_before_child_read_keeps_child_snapshot_on_fork_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork_from_main(&world.runtime, "storm");
    let child = branch_identity(&world.runtime, "storm");
    let child_root_before_main_advance = world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&child))
        .expect("child root is owner-inspectable")
        .root_ids()[0];

    advance_main_aurora_status(&mut world);

    let main = world.runtime.main_branch_identity();
    let main_root_after_advance = world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&main))
        .expect("advanced main root is owner-inspectable")
        .root_ids()[0];
    assert_ne!(
        main_root_after_advance, child_root_before_main_advance,
        "the adversarial world must contain distinct R1 and retained child R0 roots"
    );

    let child_snapshot = snapshot_for_supply_chain_identity(&world.runtime, &child);
    let child_snapshot_inspection = world
        .runtime
        .read_truth()
        .inspect_snapshot(&child_snapshot)
        .expect("child snapshot remains inspectable through the production read path");
    assert_eq!(
        child_snapshot_inspection.branch_id,
        BranchId("storm".to_owned())
    );
    assert_eq!(
        child_snapshot_inspection.root_id,
        Some(child_root_before_main_advance),
        "child read must retain R0 and never select main's newer R1"
    );
    let main_snapshot = snapshot_for_supply_chain_identity(&world.runtime, &main);
    assert_child_explicit_query_uses_snapshot_root(
        execution,
        &world,
        &child_snapshot,
        &main_snapshot,
    );
    let observed_child = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(child_snapshot.clone()),
        &world.runtime,
        &child_snapshot,
    )
    .expect("child remains semantically observable after main advances");
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Storm, None),
        &observed_child,
    )
    .expect("unchanged child truth remains exactly its fork baseline");
}

fn assert_child_explicit_query_uses_snapshot_root(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
    child_snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    advanced_main_snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) {
    let expected = world
        .runtime
        .read_truth()
        .read_snapshot(child_snapshot, execution)
        .and_then(|view| {
            view.entities()
                .iter()
                .find(|record| record.entity_id == world.handles.aurora_voyage().id)
                .cloned()
        })
        .expect("child direct read exposes Aurora from its retained root");
    let context = world
        .runtime
        .read_truth()
        .query_plan_context(child_snapshot)
        .expect("child query context remains available");
    let packet = PlannedQueryPacket::explicit_targets(
        "main-advances-before-child-explicit-query",
        context,
        vec![RecordRef::Entity(world.handles.aurora_voyage().id)],
    );
    let plan = world
        .runtime
        .read_truth()
        .plan_query_packet(child_snapshot, packet)
        .expect("child explicit query remains plan-admissible");
    let outcome = world
        .runtime
        .read_truth()
        .execute_query_plan(
            plan,
            execution,
            worth_relational::facade::runtime::RelationalQueryReadStrategy::CallingThread,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("child explicit query executes against its retained root");
    assert_eq!(outcome.result.entities, vec![expected]);

    let main_context = world
        .runtime
        .read_truth()
        .query_plan_context(advanced_main_snapshot)
        .expect("advanced main query context remains available");
    let main_packet = PlannedQueryPacket::explicit_targets(
        "advanced-main-explicit-query-control",
        main_context,
        vec![RecordRef::Entity(world.handles.aurora_voyage().id)],
    );
    let main_plan = world
        .runtime
        .read_truth()
        .plan_query_packet(advanced_main_snapshot, main_packet)
        .expect("advanced main explicit query remains plan-admissible");
    let main_outcome = world
        .runtime
        .read_truth()
        .execute_query_plan(
            main_plan,
            execution,
            worth_relational::facade::runtime::RelationalQueryReadStrategy::CallingThread,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("advanced main explicit query executes against R1");
    assert_ne!(
        outcome.result.entities, main_outcome.result.entities,
        "Aurora's main-only Held status must distinguish R1 from the child's retained R0"
    );
}

fn advance_main_aurora_status(world: &mut ProductionSeededSupplyChainWorld) {
    let status_locator = planned_single_field_locator(
        AspectKey::new("status").expect("Supply Chain status aspect"),
        FieldKey::new("status").expect("Supply Chain status field"),
    );
    let update = UpdateEntityFieldsIntent {
        entity_id: world.handles.aurora_voyage().id,
        fields: AspectFieldPatch::new(std::collections::BTreeMap::from([(
            status_locator,
            AspectValue::String(InternedString::Raw("Held".to_owned())),
        )])),
    };
    let batch = WorkerIntentBatch::new("main-advances-before-child-read").push(
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(update)),
    );
    commit_branch_batch(&world.runtime, BranchId("main".to_owned()), batch);
}

#[test]
fn phase5_named_supply_chain_deltas_keep_sibling_roots_and_work_independent() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    let scenarios = [
        ("storm", BranchLabel::Storm, DeltaId::StormRerouteAurora),
        (
            "maintenance",
            BranchLabel::Maintenance,
            DeltaId::MaintainAtlasBerth,
        ),
        (
            "medical-hold",
            BranchLabel::MedicalHold,
            DeltaId::HoldMedicalCargo,
        ),
    ];
    for (branch, _, _) in scenarios {
        fork_from_main(&world.runtime, branch);
    }
    let main_before = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .expect("main state");

    for (branch, label, delta) in scenarios {
        let selected = branch_identity(&world.runtime, branch);
        let unrelated_scopes = scenarios
            .iter()
            .filter(|(candidate, _, _)| *candidate != branch)
            .map(|(candidate, _, _)| branch_identity(&world.runtime, candidate))
            .map(|identity| RelationalMvccCostScope::capture(&world.runtime, vec![identity]))
            .collect::<Vec<_>>();
        let batch = lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &BranchId(branch.to_owned()),
            &BTreeSet::new(),
            delta,
        )
        .expect("the selected branch's actual pre-state admits its named delta");
        commit_branch_batch(&world.runtime, BranchId(branch.to_owned()), batch);

        let observed = observe_branch(execution, &mut world, &selected, branch);
        compare(
            &expected_supply_chain_branch(&world.program, label, Some(delta)),
            &observed,
        )
        .expect("production branch state matches the independently authored oracle");
        for scope in &unrelated_scopes {
            let cost = world.runtime.observe_mvcc_cost(scope).unwrap();
            assert_eq!(
                cost.sharing_cost_delta(),
                RelationalBranchSharingCostCounters::default(),
                "a selected-branch write must perform zero storage work on each sibling"
            );
        }
    }

    let identities = [
        world.runtime.main_branch_identity(),
        branch_identity(&world.runtime, "storm"),
        branch_identity(&world.runtime, "maintenance"),
        branch_identity(&world.runtime, "medical-hold"),
    ];
    let sharing = world.runtime.observe_branch_sharing(&identities).unwrap();
    assert_eq!(sharing.unique_root_count(), 4);
    for (branch, label, delta) in scenarios {
        let identity = branch_identity(&world.runtime, branch);
        let observed = observe_branch(execution, &mut world, &identity, branch);
        compare(
            &expected_supply_chain_branch(&world.program, label, Some(delta)),
            &observed,
        )
        .expect("every sibling remains semantically isolated after all writes");
    }
    let main_identity = world.runtime.main_branch_identity();
    let main_observed = observe_branch(execution, &mut world, &main_identity, "main");
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Operating, None),
        &main_observed,
    )
    .expect("main semantic truth remains unchanged after all sibling writes");
    let main_after = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .expect("main state after sibling writes");
    assert_eq!(main_after, main_before);
}

fn fork_from_main(runtime: &RelationalRuntime, branch: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    runtime
        .fork_branch(BranchId(branch.to_owned()), source)
        .expect("sibling fork shares the operating root");
}

fn branch_identity(runtime: &RelationalRuntime, branch: &str) -> RelationalBranchIdentity {
    runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("branch identity is owner-issued")
}

fn observe_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
    identity: &RelationalBranchIdentity,
    label: &str,
) -> ObservedSupplyChainState {
    let snapshot = snapshot_for_supply_chain_identity(&world.runtime, identity);
    observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap_or_else(|error| panic!("{label} branch remains observable: {error:?}"))
}
