#[allow(unused_imports)] use crate::relational_certification::ordinary_context::production_failures::*;
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::baseline_audit::audit as audit_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_budget, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::baseline_audit::BaselineAuditError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::SupplyChainProgramError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {worth_execution::ExecutionAllocationPolicy as Allocation};
pub(crate) use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::PartitionId};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntimeApi};
pub(crate) use {worth_relational::facade::schema::RelationalSchemaRegistry, worth_relational::facade::schema::SchemaRegistryErrorClass};
pub(crate) use {worth_relational::facade::symbols::ClientKey};
pub(crate) use {worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch};

#[test]
fn production_schema_installation_failure_is_typed_before_mutation() {
    let program = court_program();
    let entity_kind = entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port);
    let mut relation = program
        .schema_registry()
        .relation_registration(relation_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort))
        .expect("compiled schema has a relation kind")
        .clone();
    relation.kind_id = entity_kind;
    let entity = program
        .schema_registry()
        .entity_registration(entity_kind)
        .expect("compiled schema has an entity kind")
        .clone();

    let error = worth_relational::facade::schema::RelationalSchemaRegistry::new()
        .register_entity_kind(entity)
        .expect("entity registration is valid")
        .register_relation_kind(relation)
        .expect_err("cross-domain schema kind collision must fail registration");

    assert!(matches!(
        error.class,
        worth_relational::facade::schema::SchemaRegistryErrorClass::EntityRelationKindCollision(found) if found == entity_kind
    ));
}

#[test]
fn production_transaction_failure_is_typed_before_handle_binding() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut program = court_program();
    program.relation_specs_mut_for_test()[0].source =
        worth_relational::facade::transactions::EntityReference::Existing(worth_relational::facade::identity::EntityId::new(worth_relational::facade::identity::PartitionId::main(), u64::MAX, 0));

    let error = match compile_supply_chain_baseline(execution, program) {
        Ok(_) => panic!("a relation with an unknown endpoint must fail the commit"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError::Transaction(worth_relational::facade::transactions::TransactionCommitError::Conflict { .. })
    ));
}

#[test]
fn missing_owner_binding_is_typed_and_cannot_fall_back_to_a_raw_id() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let program = court_program();
    let owner = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .build()
        .expect("valid native runtime configuration");
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
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new("missing-owner-binding"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let commit = transaction
        .commit(execution, &owner)
        .expect("the public no-op commit supplies a real empty correspondence");
    let error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles::bind(&program, &commit, commit.snapshot.clone())
        .expect_err("a real commit with no matching owner records cannot mint semantic handles");
    assert!(matches!(error, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::MissingEntity(_)));
}

#[test]
fn foreign_snapshot_observation_is_typed_and_does_not_cross_runtime() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let program = court_program();
    let mut world =
        compile_supply_chain_baseline(execution, program).expect("Court world compiles");
    let foreign_runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .build()
        .expect("valid native runtime configuration");
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
            .commit(execution, &foreign_runtime)
            .expect("foreign runtime no-op commit");
        commit.snapshot.clone()
    };
    assert!(world
        .runtime
        .read_truth()
        .read_snapshot(&foreign_snapshot, execution)
        .expect("fixture read request remains live")
        .is_none());
    world.handles.snapshot = foreign_snapshot;

    let error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot(
        execution,
        &world.program,
        &world.handles,
        &world.runtime,
        &world.handles.snapshot,
    )
    .expect_err("a foreign runtime snapshot must not be observed");
    assert!(matches!(
        error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::SnapshotUnavailable
    ));
}

#[test]
fn relation_binding_rejects_a_snapshot_from_another_runtime() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let program = court_program();
    let world =
        compile_supply_chain_baseline(execution, program.clone()).expect("Court world compiles");
    let foreign_runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .build()
        .expect("valid native runtime configuration");
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
            .commit(execution, &foreign_runtime)
            .expect("foreign runtime no-op commit");
        commit.snapshot.clone()
    };
    let error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles::bind(&program, &world.commit_result, foreign_snapshot)
        .expect_err("owner commit and snapshot must share runtime identity");
    assert!(matches!(error, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::ForeignRuntime { .. }));
}

#[test]
fn declaration_failure_remains_distinct_from_runtime_failures() {
    let error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::empty(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court(),
    ))
    .expect_err("a Court-sized empty declaration is invalid");
    assert!(matches!(error, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::SupplyChainProgramError::Definition(_)));
}

#[test]
fn publication_budget_failure_is_typed_and_happens_before_handle_binding() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let error = match compile_supply_chain_baseline_with_budget(execution, court_program(), 1) {
        Ok(_) => panic!("an intentionally tiny publication budget must reject Court"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::SupplyChainCompilationError::Transaction(worth_relational::facade::transactions::TransactionCommitError::Publication { .. })
    ));
}

#[test]
fn semantic_relation_binding_rejects_wrong_kind_and_endpoint_intent() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let original = court_program();
    let world =
        compile_supply_chain_baseline(execution, original.clone()).expect("Court world compiles");

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
        .find(|spec| spec.client_key == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::relation_client_key(first_key))
        .expect("compiled relation spec exists")
        .kind_id = entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port);
    let wrong_kind_error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles::bind(
        &wrong_kind,
        &world.commit_result,
        world.handles.snapshot.clone(),
    )
    .expect_err("a relation kind cannot be rebound to an entity kind");
    assert!(matches!(
        wrong_kind_error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::WrongRelationKind { .. }
    ));

    let mut wrong_endpoint = original;
    let spec = wrong_endpoint
        .relation_specs_mut_for_test()
        .iter_mut()
        .find(|spec| spec.client_key == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::relation_client_key(first_key))
        .expect("compiled relation spec exists");
    spec.source =
        worth_relational::facade::transactions::EntityReference::Created(worth_relational::facade::transactions::CreatedEntityRef {
            partition_id: worth_relational::facade::identity::PartitionId::main(),
            kind_id: entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port),
            client_key: worth_relational::facade::symbols::ClientKey::raw("forged-source"),
        });
    let wrong_endpoint_error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles::bind(
        &wrong_endpoint,
        &world.commit_result,
        world.handles.snapshot.clone(),
    )
    .expect_err("relation endpoint intent cannot be silently replaced");
    assert!(matches!(
        wrong_endpoint_error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::WrongRelationEndpoints { .. }
    ));
}



#[path = "correspondence/mod.rs"]
pub(crate) mod correspondence;
#[path = "unbound_observation/mod.rs"]
pub(crate) mod unbound_observation;
