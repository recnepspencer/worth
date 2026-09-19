#[path = "structural_selection/descriptor.rs"]
mod descriptor;

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use super::invariant_oracle_expectations::expected_supply_chain_branch;
use super::world::supply_chain::{
    commit_branch_batch, compare, compile_supply_chain_baseline_with_custom_invariant,
    entity_kind_id, head_for_supply_chain_branch, lower_supply_chain_production_delta,
    observe_supply_chain, observe_supply_chain_snapshot, relation_kind_id,
    snapshot_for_supply_chain_identity, BranchLabel, CompiledSupplyChainProgram, DeltaId,
    EntityKind, RelationKind, SupplyChainScale, SupplyChainWorldDefinition,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, RelationId, VersionId};
use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantScopePlanner,
    CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion, CustomInvariantVerdict,
    InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect, InvariantGroupSet,
    InvariantReportedRule, RelationalRuntime,
};
use worth_relational::facade::transactions::{
    EntityReference, MutationIntent, RelationMutationIntent, UpdateRelationEndpointsIntent,
    WorkerIntentBatch,
};

#[test]
fn custom_invariant_structural_reads_stay_on_child_root_after_main_rewire() {
    let expectation = Arc::new(Mutex::new(None));
    let evidence = Arc::new(Mutex::new(ProbeEvidence::default()));
    let registration = CustomInvariantRegistration::new(StructuralSelectionProbe {
        expectation: Arc::clone(&expectation),
        evidence: Arc::clone(&evidence),
    })
    .expect("structural selection probe registers");
    let program = CompiledSupplyChainProgram::compile(
        SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let world = compile_supply_chain_baseline_with_custom_invariant(program, registration)
        .expect("Court world compiles with the structural selection probe");

    let relation = world.handles.relations
        [&super::world::supply_chain::RelationKey::new(RelationKind::CargoBookedOnVoyage, 0)]
        .id;
    let source = world.handles.medical_cargo().id;
    let target = world.handles.aurora_voyage().id;
    let moved_source = world.handles.entities
        [&super::world::supply_chain::EntityKey::new(EntityKind::CargoLot, 2)]
        .id;
    let moved_target = world.handles.entities
        [&super::world::supply_chain::EntityKey::new(EntityKind::Voyage, 1)]
        .id;
    let baseline = observe_supply_chain(&world).expect("baseline remains observable");
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Operating, None),
        &baseline,
    )
    .expect("production baseline matches the independent oracle");

    let (_, fork_source) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    world
        .runtime
        .fork_branch(BranchId("medical-hold".to_owned()), fork_source)
        .expect("medical-hold retains the baseline root");
    let child_basis_version =
        head_for_supply_chain_branch(&world.runtime, &BranchId("medical-hold".to_owned()))
            .version_id;

    commit_branch_batch(
        &world.runtime,
        BranchId("main".to_owned()),
        WorkerIntentBatch::new("phase5-main-rewire-cargo-booking").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: relation_kind_id(RelationKind::CargoBookedOnVoyage),
                source: EntityReference::Existing(moved_source),
                target: EntityReference::Existing(moved_target),
            }),
        )),
    );

    *expectation.lock().expect("probe expectation lock") = Some(SelectionExpectation {
        relation,
        source,
        target,
        moved_source,
        moved_target,
    });

    let child_before = observe_branch_snapshot(
        &world.program,
        &world.runtime,
        &world.handles,
        "medical-hold",
    );
    let main_after =
        observe_branch_snapshot(&world.program, &world.runtime, &world.handles, "main");
    assert_eq!(
        child_before.relations
            [&super::world::supply_chain::RelationKey::new(RelationKind::CargoBookedOnVoyage, 0,)]
            .source,
        super::world::supply_chain::EntityKey::new(EntityKind::CargoLot, 0)
    );
    assert_eq!(
        main_after.relations
            [&super::world::supply_chain::RelationKey::new(RelationKind::CargoBookedOnVoyage, 0,)]
            .source,
        super::world::supply_chain::EntityKey::new(EntityKind::CargoLot, 2)
    );
    assert_ne!(child_before.relations, main_after.relations);

    let child_batch = lower_supply_chain_production_delta(
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId("medical-hold".to_owned()),
        &BTreeSet::new(),
        DeltaId::HoldMedicalCargo,
    )
    .expect("the child medical-hold delta lowers from its selected root");
    let child_commit = commit_branch_batch_result(
        &world.runtime,
        BranchId("medical-hold".to_owned()),
        child_batch,
    );

    let child_after = observe_branch_snapshot(
        &world.program,
        &world.runtime,
        &world.handles,
        "medical-hold",
    );
    compare(
        &expected_supply_chain_branch(
            &world.program,
            BranchLabel::MedicalHold,
            Some(DeltaId::HoldMedicalCargo),
        ),
        &child_after,
    )
    .expect("child production state matches the independent medical-hold oracle");

    let custom_execution = child_commit
        .invariant_executions()
        .iter()
        .find_map(|execution| {
            execution.results().iter().find_map(|result| {
                matches!(
                    &result.rule,
                    InvariantReportedRule::Custom(identity)
                        if identity.rule_id.as_str() == StructuralSelectionProbe::RULE_ID
                )
                .then_some((execution, result))
            })
        })
        .expect("child commit records the structural selection probe");
    assert_eq!(
        custom_execution
            .1
            .custom_provenance()
            .expect("custom execution carries provenance")
            .current_version_id,
        child_basis_version
    );

    let evidence = evidence.lock().expect("probe evidence lock");
    assert_eq!(
        evidence.prepared, 1,
        "child scope is prepared once after arming"
    );
    assert_eq!(
        evidence.evaluated, 1,
        "child scope is evaluated once after arming"
    );
    assert_eq!(evidence.last_current_version, Some(child_basis_version));
    assert_eq!(evidence.last_touched_entities, 2);
    assert_eq!(evidence.last_touched_relations, 1);
    assert!(
        evidence.last_traversal_steps >= 2,
        "both bounded structural walks must charge at least the selected relation"
    );
}

fn observe_branch_snapshot(
    program: &CompiledSupplyChainProgram,
    runtime: &RelationalRuntime,
    handles: &super::world::supply_chain::SupplyChainSemanticHandles,
    branch: &str,
) -> super::world::supply_chain::ObservedSupplyChainState {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("branch identity is owner-issued");
    let snapshot = snapshot_for_supply_chain_identity(runtime, &identity);
    observe_supply_chain_snapshot(
        program,
        &handles.for_snapshot(snapshot.clone()),
        runtime,
        &snapshot,
    )
    .expect("branch snapshot remains observable")
}

fn commit_branch_batch_result(
    runtime: &RelationalRuntime,
    branch_id: BranchId,
    batch: WorkerIntentBatch,
) -> worth_relational::facade::transactions::CommitResult {
    let identity = runtime
        .branch_identity(&branch_id)
        .expect("branch identity is owner-issued");
    let options = runtime
        .admit_branch_basis(&identity)
        .expect("transaction options are owner-issued");
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction.push_batch(batch).unwrap();
    transaction
        .commit(runtime)
        .expect("branch batch commits through production publication")
}

#[derive(Clone, Copy)]
struct SelectionExpectation {
    relation: RelationId,
    source: EntityId,
    target: EntityId,
    moved_source: EntityId,
    moved_target: EntityId,
}

#[derive(Default)]
struct ProbeEvidence {
    prepared: usize,
    evaluated: usize,
    last_current_version: Option<VersionId>,
    last_touched_entities: usize,
    last_touched_relations: usize,
    last_traversal_steps: usize,
}

struct StructuralSelectionProbe {
    expectation: Arc<Mutex<Option<SelectionExpectation>>>,
    evidence: Arc<Mutex<ProbeEvidence>>,
}

macro_rules! assert_selected_views {
    ($touched:expr, $relations:expr, $traversal:expr, $expected:expr) => {{
        let touched = $touched;
        let relations = $relations;
        let traversal = $traversal;
        let expected = $expected;
        let mut expected_entities = vec![expected.source, expected.target];
        expected_entities.sort_unstable();
        assert_eq!(touched.visible_entity_ids(), expected_entities.as_slice());
        assert_eq!(touched.visible_relation_ids(), &[expected.relation]);
        let mut expected_partitions = vec![
            expected.source.partition_id,
            expected.target.partition_id,
            expected.relation.partition_id,
        ];
        expected_partitions.sort_unstable();
        expected_partitions.dedup();
        assert_eq!(touched.touched_partitions(), expected_partitions.as_slice());
        assert!(
            !touched
                .visible_entity_ids()
                .contains(&expected.moved_source),
            "selected touched scope must exclude main-only moved source"
        );
        assert!(
            !touched
                .visible_entity_ids()
                .contains(&expected.moved_target),
            "selected touched scope must exclude main-only moved target"
        );
        let record = relations
            .relation(expected.relation)
            .expect("selected relation remains visible");
        assert_eq!(record.source, expected.source);
        assert_eq!(record.target, expected.target);
        assert!(relations
            .outgoing_relations_for_entity(expected.source)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        assert!(relations
            .incoming_relations_for_entity(expected.target)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        assert!(relations
            .all_relations_for_entity(expected.source)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        assert!(!relations
            .outgoing_relations_for_entity(expected.moved_source)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        assert!(!relations
            .incoming_relations_for_entity(expected.moved_target)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        let outgoing = traversal
            .walk_outgoing_from(&[expected.source], 1)
            .expect("selected outgoing traversal remains bounded");
        assert!(outgoing.traversed_relations().contains(&expected.relation));
        assert!(outgoing.visited_entities().contains(&expected.target));
        let incoming = traversal
            .walk_incoming_from(&[expected.target], 1)
            .expect("selected incoming traversal remains bounded");
        assert!(incoming.traversed_relations().contains(&expected.relation));
        assert!(incoming.visited_entities().contains(&expected.source));
    }};
}

impl StructuralSelectionProbe {
    const RULE_ID: &'static str = "supply_chain.selected-structural-adjacency";

    fn expectation(&self) -> Option<SelectionExpectation> {
        self.expectation
            .lock()
            .expect("probe expectation lock")
            .as_ref()
            .copied()
    }
}

impl CustomInvariantRule for StructuralSelectionProbe {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        descriptor::selection_descriptor()
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let Some(expected) = self.expectation() else {
            return Ok(());
        };
        assert_selected_views!(
            planner.touched(),
            planner.relations(),
            planner.traversal(),
            expected
        );
        self.evidence.lock().expect("probe evidence lock").prepared += 1;
        Ok(())
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let Some(expected) = self.expectation() else {
            return Ok(CustomInvariantVerdict::Pass);
        };
        assert_selected_views!(
            context.touched(),
            context.relations(),
            context.traversal(),
            expected
        );
        let provenance = context.provenance();
        let mut evidence = self.evidence.lock().expect("probe evidence lock");
        evidence.evaluated += 1;
        evidence.last_current_version = Some(provenance.current_version_id);
        evidence.last_touched_entities = provenance.counts.visible_entity_count();
        evidence.last_touched_relations = provenance.counts.visible_relation_count();
        evidence.last_traversal_steps = provenance.traversal.consumed_steps;
        Ok(CustomInvariantVerdict::Pass)
    }
}
