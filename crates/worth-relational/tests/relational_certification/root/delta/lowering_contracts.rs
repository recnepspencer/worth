use std::collections::{BTreeMap, BTreeSet};

use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
use super::world::supply_chain::{
    commit_branch_batch, lower_supply_chain_production_delta, observe_supply_chain_snapshot,
    snapshot_for_supply_chain_identity, BookingStatus, BranchLabel, DeltaId, EntityKey, EntityKind,
    EntityRecord, SupplyChainProductionDeltaLoweringError, SupplyChainScale,
};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, EntityMutationIntent, MutationIntent,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};

#[test]
fn phase5_lowering_reads_actual_branch_pre_state_instead_of_expected_oracle() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(&world.runtime, "storm");
    let voyage = EntityKey::new(EntityKind::Voyage, 0);
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("storm".to_owned()),
        update_number(&world.handles, voyage, "arrival", 9_000),
    );

    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId("storm".to_owned()),
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .expect("actual pre-state satisfies the Storm contract");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("storm".to_owned()),
        batch,
    );

    let observed = observe_branch(execution, &mut world, "storm");
    let EntityRecord::Voyage(voyage) = &observed.entities[&voyage] else {
        panic!("Aurora voyage keeps its owner-observed kind");
    };
    assert_eq!(voyage.arrival.0, 9_030);
    let EntityRecord::Voyage(oracle_voyage) =
        &baseline.entities[&EntityKey::new(EntityKind::Voyage, 0)]
    else {
        panic!("oracle baseline declares Aurora voyage");
    };
    assert_ne!(
        voyage.arrival.0,
        oracle_voyage.arrival.0 + 30,
        "the oracle baseline cannot feed production intent lowering"
    );
}

#[test]
fn phase5_lowering_rejects_wrong_branch_and_duplicate_delta_preconditions() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(&world.runtime, "storm");
    assert!(matches!(
        lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &BranchId("storm".to_owned()),
            &BTreeSet::new(),
            DeltaId::MaintainAtlasBerth,
        ),
        Err(SupplyChainProductionDeltaLoweringError::WrongBranch {
            expected: BranchLabel::Maintenance,
            observed: BranchLabel::Storm,
        })
    ));
    let previously_applied = BTreeSet::from([DeltaId::StormRerouteAurora]);
    assert!(matches!(
        lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &BranchId("storm".to_owned()),
            &previously_applied,
            DeltaId::StormRerouteAurora,
        ),
        Err(SupplyChainProductionDeltaLoweringError::DuplicateDelta(
            DeltaId::StormRerouteAurora
        ))
    ));
}

#[test]
fn maintenance_rewire_and_medical_lower_from_each_actual_branch_pre_state() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    prove_number_prestate(
        execution,
        NumberPrestateCase {
            branch: "maintenance",
            entity: EntityKey::new(EntityKind::Voyage, 0),
            field: "arrival",
            prestate: 12_000,
            delta: DeltaId::MaintainAtlasBerth,
            observe: |record| match record {
                EntityRecord::Voyage(voyage) => voyage.arrival.0 as u64,
                _ => panic!("Aurora voyage keeps its kind"),
            },
            expected: 12_060,
        },
    );
    prove_number_prestate(
        execution,
        NumberPrestateCase {
            branch: "rewire",
            entity: EntityKey::new(EntityKind::PortCall, 1),
            field: "revision",
            prestate: 40,
            delta: DeltaId::RewireAuroraPortCall,
            observe: |record| match record {
                EntityRecord::PortCall(call) => call.revision as u64,
                _ => panic!("Aurora port call keeps its kind"),
            },
            expected: 41,
        },
    );

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(&world.runtime, "medical-hold");
    let cargo = EntityKey::new(EntityKind::CargoLot, 0);
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("medical-hold".to_owned()),
        update_text(&world.handles, cargo, "booking", "Available"),
    );
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId("medical-hold".to_owned()),
        &BTreeSet::new(),
        DeltaId::HoldMedicalCargo,
    )
    .expect("Medical Hold is an absolute transition over the actual cargo row");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("medical-hold".to_owned()),
        batch,
    );
    let EntityRecord::CargoLot(cargo) =
        &observe_branch(execution, &mut world, "medical-hold").entities[&cargo]
    else {
        panic!("medical cargo keeps its kind");
    };
    assert_eq!(cargo.booking, BookingStatus::Held);
}

struct NumberPrestateCase {
    branch: &'static str,
    entity: EntityKey,
    field: &'static str,
    prestate: u64,
    delta: DeltaId,
    observe: fn(&EntityRecord) -> u64,
    expected: u64,
}

fn prove_number_prestate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    case: NumberPrestateCase,
) {
    let NumberPrestateCase {
        branch,
        entity,
        field,
        prestate,
        delta,
        observe,
        expected,
    } = case;
    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(&world.runtime, branch);
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId(branch.to_owned()),
        update_number(&world.handles, entity, field, prestate),
    );
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId(branch.to_owned()),
        &BTreeSet::new(),
        delta,
    )
    .expect("actual branch pre-state admits its semantic delta");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId(branch.to_owned()),
        batch,
    );
    assert_eq!(
        observe(&observe_branch(execution, &mut world, branch).entities[&entity]),
        expected
    );
}

fn fork(runtime: &worth_relational::facade::runtime::RelationalRuntime, branch: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime
        .fork_branch(BranchId(branch.to_owned()), source)
        .unwrap();
}

fn update_number(
    handles: &super::world::supply_chain::SupplyChainSemanticHandles,
    entity: EntityKey,
    field: &str,
    value: u64,
) -> WorkerIntentBatch {
    let mut fields = BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            AspectKey::new(field).unwrap(),
            FieldKey::new(field).unwrap(),
        ),
        AspectValue::UInt64(value),
    );
    WorkerIntentBatch::new("phase5-actual-prestate-control").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: handles.entities[&entity].id,
            fields: AspectFieldPatch::new(fields),
        }),
    ))
}

fn update_text(
    handles: &super::world::supply_chain::SupplyChainSemanticHandles,
    entity: EntityKey,
    field: &str,
    value: &str,
) -> WorkerIntentBatch {
    let mut fields = BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            AspectKey::new(field).unwrap(),
            FieldKey::new(field).unwrap(),
        ),
        AspectValue::String(worth_foundational::facade::InternedString::Raw(
            value.to_owned(),
        )),
    );
    WorkerIntentBatch::new("phase5-actual-text-prestate-control").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: handles.entities[&entity].id,
            fields: AspectFieldPatch::new(fields),
        }),
    ))
}

fn observe_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
    branch: &str,
) -> super::world::supply_chain::ObservedSupplyChainState {
    let identity = world
        .runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .unwrap();
    let snapshot = snapshot_for_supply_chain_identity(&world.runtime, &identity);
    observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap()
}
