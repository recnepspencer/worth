pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline, crate::relational_certification::ordinary_context::production_failures::court_program};

#[test]
fn observation_rejects_missing_snapshot_and_unbound_record_identities() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let missing_snapshot =
        compile_supply_chain_baseline(execution, court_program()).expect("Court world compiles");
    let released_snapshot = missing_snapshot.handles.snapshot.clone();
    assert!(missing_snapshot
        .runtime
        .snapshots()
        .release_snapshot(&released_snapshot)
        .is_ok());
    let missing_error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot(
        execution,
        &missing_snapshot.program,
        &missing_snapshot.handles,
        &missing_snapshot.runtime,
        &released_snapshot,
    )
    .expect_err("an unknown snapshot must not fall back to current state");
    assert!(matches!(
        missing_error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::SnapshotUnavailable
    ));

    let mut unknown_entity =
        compile_supply_chain_baseline(execution, court_program()).expect("Court world compiles");
    let entity_key = *unknown_entity
        .handles
        .entities
        .keys()
        .next()
        .expect("Court has an entity");
    unknown_entity.handles.entities.remove(&entity_key);
    let entity_error =
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe(execution, &unknown_entity)
            .expect_err("an unbound production entity must not be ignored");
    assert!(matches!(
        entity_error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::UnknownEntityIdentity(_)
    ));

    let mut unknown_relation =
        compile_supply_chain_baseline(execution, court_program()).expect("Court world compiles");
    let relation_key = *unknown_relation.handles.relations.keys().next().unwrap();
    unknown_relation.handles.relations.remove(&relation_key);
    let relation_error =
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe(execution, &unknown_relation)
            .expect_err("an unbound production relation must not be ignored");
    assert!(matches!(
        relation_error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::UnknownRelationIdentity(_)
    ));
}
