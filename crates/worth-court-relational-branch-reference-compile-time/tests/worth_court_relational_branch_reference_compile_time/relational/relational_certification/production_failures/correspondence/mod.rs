pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline, crate::relational_certification::ordinary_context::production_failures::court_program};

#[test]
fn semantic_relation_binding_rejects_incomplete_correspondence() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let original = court_program();
    let world =
        compile_supply_chain_baseline(execution, original.clone()).expect("Court world compiles");
    let first_key = *original
        .definition()
        .relations
        .keys()
        .find(|key| key.kind == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone)
        .expect("Court has the repeated pilotage relation");

    let mut incomplete = original.clone();
    incomplete.relation_specs_mut_for_test().retain(|spec| {
        spec.client_key != worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::relation_client_key(first_key)
    });
    let incomplete_error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles::bind(
        &incomplete,
        &world.commit_result,
        world.handles.snapshot.clone(),
    )
    .expect_err("missing semantic relation spec must be typed");
    assert!(matches!(
        incomplete_error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::MissingRelationSpec(_)
    ));
}

#[test]
fn semantic_relation_binding_rejects_duplicate_correspondence() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let original = court_program();
    let world =
        compile_supply_chain_baseline(execution, original.clone()).expect("Court world compiles");
    let keys = original
        .definition()
        .relations
        .keys()
        .filter(|key| key.kind == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone)
        .copied()
        .collect::<Vec<_>>();
    let mut duplicate = original;
    let first_spec = duplicate
        .relation_specs()
        .iter()
        .find(|spec| {
            spec.client_key == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::relation_client_key(keys[0])
        })
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
        .find(|spec| {
            spec.client_key == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::relation_client_key(keys[1])
        })
        .expect("second repeated relation spec exists")
        .clone_from(&first_spec);
    let duplicate_error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles::bind(
        &duplicate,
        &world.commit_result,
        world.handles.snapshot.clone(),
    )
    .expect_err("two semantic relations cannot claim one owner identity");
    assert!(matches!(
        duplicate_error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::HandleBindingError::DuplicateRelationReference(_)
    ));
}
