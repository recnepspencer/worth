use super::world::supply_chain::relation_kind_id;
use super::world::supply_chain::{
    audit_supply_chain_baseline, compile_supply_chain_baseline,
    compile_supply_chain_baseline_with_budget, entity_kind_id, BaselineAuditError,
    CompiledSupplyChainProgram, HandleBindingError, SupplyChainCompilationError,
    SupplyChainProgramError, SupplyChainScale, SupplyChainSemanticHandles,
    SupplyChainWorldDefinition,
};
use super::world::supply_chain::{
    ComparisonMismatch, EntityKey, EntityKind, EntityRecord, RelationKind,
};
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaRegistryErrorClass};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    EntityReference, TransactionCommitError, WorkerIntentBatch,
};

pub(super) fn court_program() -> CompiledSupplyChainProgram {
    CompiledSupplyChainProgram::compile(
        SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Court Supply Chain program compiles")
}

#[test]
fn production_schema_installation_failure_is_typed_before_mutation() {
    let program = court_program();
    let entity_kind = entity_kind_id(EntityKind::Port);
    let mut relation = program
        .schema_registry()
        .relation_registration(relation_kind_id(RelationKind::TerminalAtPort))
        .expect("compiled schema has a relation kind")
        .clone();
    relation.kind_id = entity_kind;
    let entity = program
        .schema_registry()
        .entity_registration(entity_kind)
        .expect("compiled schema has an entity kind")
        .clone();

    let error = RelationalSchemaRegistry::new()
        .register_entity_kind(entity)
        .expect("entity registration is valid")
        .register_relation_kind(relation)
        .expect_err("cross-domain schema kind collision must fail registration");

    assert!(matches!(
        error.class,
        SchemaRegistryErrorClass::EntityRelationKindCollision(found) if found == entity_kind
    ));
}

#[test]
fn production_transaction_failure_is_typed_before_handle_binding() {
    let mut program = court_program();
    program.relation_specs_mut_for_test()[0].source =
        EntityReference::Existing(EntityId::new(PartitionId::main(), u64::MAX, 0));

    let error = match compile_supply_chain_baseline(program) {
        Ok(_) => panic!("a relation with an unknown endpoint must fail the commit"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        SupplyChainCompilationError::Transaction(TransactionCommitError::Conflict { .. })
    ));
}

#[test]
fn missing_owner_binding_is_typed_and_cannot_fall_back_to_a_raw_id() {
    let program = court_program();
    let owner = RelationalRuntimeApi::builder().build();
    let owner_identity = owner.main_branch_identity();
    let owner_options = owner
        .admit_branch_basis(&owner_identity)
        .expect("configured main branch must remain owner-admissible");
    let mut transaction = owner
        .begin_branch_transaction(
            &owner_options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(WorkerIntentBatch::new("missing-owner-binding"))
        .unwrap();
    let commit = transaction
        .commit(&owner)
        .expect("the public no-op commit supplies a real empty correspondence");
    let error = SupplyChainSemanticHandles::bind(&program, &commit, commit.snapshot.clone())
        .expect_err("a real commit with no matching owner records cannot mint semantic handles");
    assert!(matches!(error, HandleBindingError::MissingEntity(_)));
}

#[test]
fn foreign_snapshot_observation_is_typed_and_does_not_cross_runtime() {
    let program = court_program();
    let mut world = compile_supply_chain_baseline(program).expect("Court world compiles");
    let foreign_runtime = RelationalRuntimeApi::builder().build();
    let foreign_snapshot = {
        let identity = foreign_runtime.main_branch_identity();
        let options = foreign_runtime
            .admit_branch_basis(&identity)
            .expect("foreign runtime owner binding");
        let transaction = foreign_runtime
            .begin_branch_transaction(
                &options,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context");
        let commit = transaction
            .commit(&foreign_runtime)
            .expect("foreign runtime no-op commit");
        commit.snapshot.clone()
    };
    assert!(world
        .runtime
        .read_truth()
        .read_snapshot(&foreign_snapshot)
        .is_none());
    world.handles.snapshot = foreign_snapshot;

    let error = super::world::supply_chain::observe_supply_chain_snapshot(
        &world.program,
        &world.handles,
        &world.runtime,
        &world.handles.snapshot,
    )
    .expect_err("a foreign runtime snapshot must not be observed");
    assert!(matches!(
        error,
        super::world::supply_chain::ObservationError::SnapshotUnavailable
    ));
}

#[test]
fn relation_binding_rejects_a_snapshot_from_another_runtime() {
    let program = court_program();
    let world = compile_supply_chain_baseline(program.clone()).expect("Court world compiles");
    let foreign_runtime = RelationalRuntimeApi::builder().build();
    let foreign_snapshot = {
        let identity = foreign_runtime.main_branch_identity();
        let options = foreign_runtime
            .admit_branch_basis(&identity)
            .expect("foreign runtime owner binding");
        let transaction = foreign_runtime
            .begin_branch_transaction(
                &options,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context");
        let commit = transaction
            .commit(&foreign_runtime)
            .expect("foreign runtime no-op commit");
        commit.snapshot.clone()
    };
    let error = SupplyChainSemanticHandles::bind(&program, &world.commit_result, foreign_snapshot)
        .expect_err("owner commit and snapshot must share runtime identity");
    assert!(matches!(error, HandleBindingError::ForeignRuntime { .. }));
}

#[test]
fn oracle_failure_is_distinct_from_production_observation_and_comparison() {
    let mut world = compile_supply_chain_baseline(court_program()).expect("Court world compiles");
    let edge = world
        .program
        .definition_mut_for_test()
        .relations
        .values_mut()
        .next()
        .expect("Court world has a relation");
    edge.source = EntityKey::new(EntityKind::Port, u32::MAX);

    let error = match audit_supply_chain_baseline(world) {
        Ok(_) => panic!("invalid oracle topology is rejected"),
        Err(error) => error,
    };
    assert!(matches!(error, BaselineAuditError::Oracle(_)));
}

#[test]
fn comparison_failure_is_distinct_after_an_oracle_only_value_drift() {
    let mut world = compile_supply_chain_baseline(court_program()).expect("Court world compiles");
    let port = world
        .program
        .definition_mut_for_test()
        .entities
        .get_mut(&EntityKey::new(EntityKind::Port, 0))
        .expect("Court world has the named first port");
    match port {
        EntityRecord::Port(record) => record.name.push_str("-oracle-only-drift"),
        other => panic!("expected a Port record, got {other:?}"),
    }

    let error = match audit_supply_chain_baseline(world) {
        Ok(_) => panic!("oracle drift must fail comparison"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        BaselineAuditError::Comparison(ComparisonMismatch::EntityValue(_))
    ));
}

#[test]
fn declaration_failure_remains_distinct_from_runtime_failures() {
    let error = CompiledSupplyChainProgram::compile(SupplyChainWorldDefinition::empty(
        SupplyChainScale::court(),
    ))
    .expect_err("a Court-sized empty declaration is invalid");
    assert!(matches!(error, SupplyChainProgramError::Definition(_)));
}

#[test]
fn publication_budget_failure_is_typed_and_happens_before_handle_binding() {
    let error = match compile_supply_chain_baseline_with_budget(court_program(), 1) {
        Ok(_) => panic!("an intentionally tiny publication budget must reject Court"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        SupplyChainCompilationError::Transaction(TransactionCommitError::Publication { .. })
    ));
}

#[test]
fn semantic_relation_binding_rejects_wrong_kind_and_endpoint_intent() {
    let original = court_program();
    let world = compile_supply_chain_baseline(original.clone()).expect("Court world compiles");

    let mut wrong_kind = original.clone();
    let first_key = *wrong_kind
        .definition()
        .relations
        .keys()
        .next()
        .expect("Court has a relation");
    wrong_kind
        .relation_specs_mut_for_test()
        .iter_mut()
        .find(|spec| spec.client_key == super::world::supply_chain::relation_client_key(first_key))
        .expect("compiled relation spec exists")
        .kind_id = entity_kind_id(EntityKind::Port);
    let wrong_kind_error = SupplyChainSemanticHandles::bind(
        &wrong_kind,
        &world.commit_result,
        world.handles.snapshot.clone(),
    )
    .expect_err("a relation kind cannot be rebound to an entity kind");
    assert!(matches!(
        wrong_kind_error,
        HandleBindingError::WrongRelationKind { .. }
    ));

    let mut wrong_endpoint = original;
    let spec = wrong_endpoint
        .relation_specs_mut_for_test()
        .iter_mut()
        .find(|spec| spec.client_key == super::world::supply_chain::relation_client_key(first_key))
        .expect("compiled relation spec exists");
    spec.source =
        EntityReference::Created(worth_relational::facade::transactions::CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: entity_kind_id(EntityKind::Port),
            client_key: ClientKey::raw("forged-source"),
        });
    let wrong_endpoint_error = SupplyChainSemanticHandles::bind(
        &wrong_endpoint,
        &world.commit_result,
        world.handles.snapshot.clone(),
    )
    .expect_err("relation endpoint intent cannot be silently replaced");
    assert!(matches!(
        wrong_endpoint_error,
        HandleBindingError::WrongRelationEndpoints { .. }
    ));
}

#[test]
fn semantic_relation_binding_rejects_incomplete_correspondence() {
    let original = court_program();
    let world = compile_supply_chain_baseline(original.clone()).expect("Court world compiles");
    let first_key = *original
        .definition()
        .relations
        .keys()
        .find(|key| key.kind == RelationKind::SharesPilotageZone)
        .expect("Court has the repeated pilotage relation");

    let mut incomplete = original.clone();
    incomplete.relation_specs_mut_for_test().retain(|spec| {
        spec.client_key != super::world::supply_chain::relation_client_key(first_key)
    });
    let incomplete_error = SupplyChainSemanticHandles::bind(
        &incomplete,
        &world.commit_result,
        world.handles.snapshot.clone(),
    )
    .expect_err("missing semantic relation spec must be typed");
    assert!(matches!(
        incomplete_error,
        HandleBindingError::MissingRelationSpec(_)
    ));
}

#[test]
fn semantic_relation_binding_rejects_duplicate_correspondence() {
    let original = court_program();
    let world = compile_supply_chain_baseline(original.clone()).expect("Court world compiles");
    let keys = original
        .definition()
        .relations
        .keys()
        .filter(|key| key.kind == RelationKind::SharesPilotageZone)
        .copied()
        .collect::<Vec<_>>();
    let mut duplicate = original;
    let first_spec = duplicate
        .relation_specs()
        .iter()
        .find(|spec| spec.client_key == super::world::supply_chain::relation_client_key(keys[0]))
        .expect("first repeated relation spec exists")
        .clone();
    let first_edge = duplicate.definition().relations[&keys[0]];
    duplicate
        .definition_mut_for_test()
        .relations
        .insert(keys[1], first_edge);
    duplicate
        .relation_specs_mut_for_test()
        .iter_mut()
        .find(|spec| spec.client_key == super::world::supply_chain::relation_client_key(keys[1]))
        .expect("second repeated relation spec exists")
        .clone_from(&first_spec);
    let duplicate_error = SupplyChainSemanticHandles::bind(
        &duplicate,
        &world.commit_result,
        world.handles.snapshot.clone(),
    )
    .expect_err("two semantic relations cannot claim one owner identity");
    assert!(matches!(
        duplicate_error,
        HandleBindingError::DuplicateRelationReference(_)
    ));
}

#[test]
fn observation_rejects_missing_snapshot_and_unbound_record_identities() {
    let missing_snapshot =
        compile_supply_chain_baseline(court_program()).expect("Court world compiles");
    let released_snapshot = missing_snapshot.handles.snapshot.clone();
    assert!(missing_snapshot
        .runtime
        .snapshots()
        .release_snapshot(&released_snapshot)
        .is_ok());
    let missing_error = super::world::supply_chain::observe_supply_chain_snapshot(
        &missing_snapshot.program,
        &missing_snapshot.handles,
        &missing_snapshot.runtime,
        &released_snapshot,
    )
    .expect_err("an unknown snapshot must not fall back to current state");
    assert!(matches!(
        missing_error,
        super::world::supply_chain::ObservationError::SnapshotUnavailable
    ));

    let mut unknown_entity =
        compile_supply_chain_baseline(court_program()).expect("Court world compiles");
    let entity_key = *unknown_entity
        .handles
        .entities
        .keys()
        .next()
        .expect("Court has an entity");
    unknown_entity.handles.entities.remove(&entity_key);
    let entity_error = super::world::supply_chain::observe_supply_chain(&unknown_entity)
        .expect_err("an unbound production entity must not be ignored");
    assert!(matches!(
        entity_error,
        super::world::supply_chain::ObservationError::UnknownEntityIdentity(_)
    ));

    let mut unknown_relation =
        compile_supply_chain_baseline(court_program()).expect("Court world compiles");
    let relation_key = *unknown_relation.handles.relations.keys().next().unwrap();
    unknown_relation.handles.relations.remove(&relation_key);
    let relation_error = super::world::supply_chain::observe_supply_chain(&unknown_relation)
        .expect_err("an unbound production relation must not be ignored");
    assert!(matches!(
        relation_error,
        super::world::supply_chain::ObservationError::UnknownRelationIdentity(_)
    ));
}
