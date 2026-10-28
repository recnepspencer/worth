pub(crate) use std::collections::BTreeSet;

pub(crate) use super::invariant_oracle_expectations::expected_supply_chain_branch;
pub(crate) use super::world::supply_chain::{
    assert_oracle_matches, certified_supply_chain_world, commit_supply_chain_delta, compare,
    entity_kind_id, hazard_v2_transition, lower_supply_chain_production_delta,
    observe_supply_chain_snapshot, schema_registry_with_altered_port_contract, BranchLabel,
    DeltaId, EntityKind, SchemaVersion, SupplyChainScale,
};
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_relational::facade::replay::{
    RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode,
};
pub(crate) use worth_relational::facade::runtime::RelationalSchemaTransitionAdmissionDenialKind;
pub(crate) use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};

#[test]
fn branch_schema_target_admission_is_typed_and_never_moves_a_branch() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    let main = BranchId("main".to_owned());
    let before = world.runtime.branch_reference_state(&main).unwrap();
    let identity = world.runtime.main_branch_identity();
    let basis = world.runtime.admit_branch_basis(&identity).unwrap();

    let mut wrong_source = hazard_v2_transition();
    wrong_source.source_schema_version_id = SchemaVersionId(7);
    let denial = world
        .runtime
        .begin_branch_schema_transition(
            &basis,
            wrong_source,
            None,
            world
                .program
                .schema_registry_for_version(SchemaVersion::V2)
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::SourceBasisMismatch
    );
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);

    let altered_target = schema_registry_with_altered_port_contract(SchemaVersionId(2)).unwrap();
    let denial = world
        .runtime
        .begin_branch_schema_transition(&basis, hazard_v2_transition(), None, altered_target)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::TargetContractMismatch
    );
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);

    let partial_target = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: entity_kind_id(EntityKind::Port),
            kind_name: "supply_chain.entity.Port".to_owned(),
            schema_id: SchemaId("supply_chain".to_owned()),
            schema_version_id: SchemaVersionId(2),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let denial = world
        .runtime
        .begin_branch_schema_transition(&basis, hazard_v2_transition(), None, partial_target)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::TargetContractMismatch
    );
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);

    let transition = hazard_v2_transition();
    let denial = world
        .runtime
        .begin_branch_schema_transition(
            &basis,
            transition.clone(),
            None,
            world
                .program
                .schema_registry_for_version(SchemaVersion::V1)
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::TargetBasisMismatch
    );
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);

    let transaction = world
        .runtime
        .begin_branch_schema_transition(
            &basis,
            transition,
            None,
            world
                .program
                .schema_registry_for_version(SchemaVersion::V2)
                .unwrap(),
        )
        .unwrap();
    drop(transaction);
    assert_eq!(world.runtime.branch_reference_state(&main).unwrap(), before);
}

#[test]
fn stale_schema_transition_basis_cannot_install_target_authority() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let basis = world.runtime.admit_branch_basis(&identity).unwrap();
    let ordinary = world
        .runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    let committed = ordinary.commit(execution, &world.runtime).unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();

    let denial = world
        .runtime
        .begin_branch_schema_transition(
            &basis,
            hazard_v2_transition(),
            None,
            world
                .program
                .schema_registry_for_version(SchemaVersion::V2)
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::TransactionAdmission
    );
    let fresh_basis = world.runtime.admit_branch_basis(&identity).unwrap();
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&fresh_basis.observation())
        .unwrap();
    let observed = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap();
    assert_eq!(observed.schema, SchemaVersion::V1);
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
}
