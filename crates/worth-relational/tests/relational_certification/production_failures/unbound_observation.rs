use super::*;

#[test]
fn observation_rejects_missing_snapshot_and_unbound_record_identities() {
    let host_request = crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let missing_snapshot =
        compile_supply_chain_baseline(execution, court_program()).expect("Court world compiles");
    let released_snapshot = missing_snapshot.handles.snapshot.clone();
    assert!(missing_snapshot
        .runtime
        .snapshots()
        .release_snapshot(&released_snapshot)
        .is_ok());
    let missing_error = super::super::world::supply_chain::observe_supply_chain_snapshot(
        execution,
        &missing_snapshot.program,
        &missing_snapshot.handles,
        &missing_snapshot.runtime,
        &released_snapshot,
    )
    .expect_err("an unknown snapshot must not fall back to current state");
    assert!(matches!(
        missing_error,
        super::super::world::supply_chain::ObservationError::SnapshotUnavailable
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
        super::super::world::supply_chain::observe_supply_chain(execution, &unknown_entity)
            .expect_err("an unbound production entity must not be ignored");
    assert!(matches!(
        entity_error,
        super::super::world::supply_chain::ObservationError::UnknownEntityIdentity(_)
    ));

    let mut unknown_relation =
        compile_supply_chain_baseline(execution, court_program()).expect("Court world compiles");
    let relation_key = *unknown_relation.handles.relations.keys().next().unwrap();
    unknown_relation.handles.relations.remove(&relation_key);
    let relation_error =
        super::super::world::supply_chain::observe_supply_chain(execution, &unknown_relation)
            .expect_err("an unbound production relation must not be ignored");
    assert!(matches!(
        relation_error,
        super::super::world::supply_chain::ObservationError::UnknownRelationIdentity(_)
    ));
}
