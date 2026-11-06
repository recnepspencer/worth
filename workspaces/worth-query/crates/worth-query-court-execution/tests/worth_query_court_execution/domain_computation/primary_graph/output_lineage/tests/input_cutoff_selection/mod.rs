use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

use worth_runtime_world::facade::ProductBranchCreationIntent;
use worth_runtime_world::facade::ProductBranchCreationPlans;
use worth_runtime_world::facade::RelationalBranchCreationPlan;
use worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome;
use worth_runtime_world::facade::RuntimeWorldCancellationSource;
use worth_runtime_world::facade::SignalBranchCreationPlan;

use crate::domain_computation::primary_graph::output_lineage::tests::checkpoint_identity;
use crate::domain_computation::primary_graph::output_lineage::tests::RestoredOutputBinding;
use worth_query_execution::facade::domain_computation::WorthQueryOperationScopeBinding;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::ProductCoordinate;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;

#[test]
fn cutoff_candidate_pins_the_selected_ancestor_partition() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let product = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("a real product occurrence is live");
    let parent = product.observation_for_test();
    let intent = ProductBranchCreationIntent::from_source(
        "cutoff-ancestor",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .expect("the fork intent is valid");
    let RuntimeWorldBranchCreationOutcome::Performed(child) = world
        .application
        .product_runtime()
        .create_product_branch_for_test(
            execution,
            &product,
            None,
            intent,
            &RuntimeWorldCancellationSource::new().token(),
        )
        .expect("the product owner admits the fork")
    else {
        panic!("the product owner must create the fork");
    };

    let runtime_authority = (*world.application.runtime_for_test())
        .authority_identity_for_test()
        .as_u64();
    let schema = (*world.application.installed_schema_for_test()).binding_identity();
    let scope_entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let scope = WorthQueryOperationScopeBinding::axis_probe_scope_for_test(
        runtime_authority,
        schema.clone(),
        "cutoff-selection",
        scope_entity.partition_value(),
        scope_entity.local_slot_value(),
        scope_entity.generation_value(),
        scope_entity.partition_value(),
        scope_entity.local_slot_value(),
        scope_entity.generation_value(),
    );
    let source = SemanticSource::new_for_test(
        runtime_authority,
        schema.clone(),
        scope.scope(),
        OutputBindingIdentity::declared_for_test("RestoredOutputBinding"),
    );
    let target_partition = [0x11; 32];
    let sibling_partition = [0x22; 32];
    let target = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let sibling = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        "RestoredOutputBinding",
    );

    lineage.record_recovered_prior_output_for_test(
        lineage.binding_type_for_test(source.output_binding_for_test()),
        runtime_authority,
        schema.clone(),
        source.scope,
        parent.lifecycle_incarnation(),
        parent.reference_generation().get(),
        Arc::clone(&target),
        checkpoint_identity([0x31; 32]),
        target_partition,
        None,
        [0x41; 32],
        None,
    );
    lineage.record_recovered_prior_output_for_test(
        lineage.binding_type_for_test(source.output_binding_for_test()),
        runtime_authority,
        schema,
        source.scope,
        parent.lifecycle_incarnation(),
        parent.reference_generation().get(),
        sibling,
        checkpoint_identity([0x32; 32]),
        sibling_partition,
        None,
        [0x42; 32],
        None,
    );
    lineage.register_fork_for_test(parent, &child);
    let selected_cell = &(*lineage.by_source_for_test())[&source][&parent.lifecycle_incarnation()]
        [&parent.reference_generation().get()][0];
    let selected_identity = selected_cell.get().unwrap().settlement_identity_for_test();
    assert!(matches!((*selected_cell.get().unwrap().mutable_for_test()).lock().unwrap().computation_for_test(),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation::Absent(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence::Restored)));

    let selected = lineage
        .prior_input_cutoff_candidate_for_test::<RestoredOutputBinding>(
            &scope,
            &child,
            target_partition,
            execution,
        )
        .expect("the selected ancestor lookup selects exact custody")
        .expect("the matching parent partition is retained");
    assert!(Arc::ptr_eq(
        selected.settlement_identity_for_test(),
        selected_identity
    ));
    assert!(
        selected.prepared_input_key_for_test().is_none(),
        "restored rows cannot cut off"
    );

    let next_generation = parent.reference_generation().get() + 1;
    let _unpublished = (*lineage.partition_index_mut_for_test()).insert_vacancy_for_test(
        source.clone(),
        ProductCoordinate::new_for_test(parent.lifecycle_incarnation(), next_generation),
        Some(target_partition),
    );
    let prior = (*lineage.partition_index_for_test())
        .latest_admitted_for_test(
            execution,
            &source,
            ProductCoordinate::new_for_test(parent.lifecycle_incarnation(), next_generation),
            target_partition,
        )
        .expect("the newer empty slot is a bounded lookup");
    assert_eq!(prior, Some((parent.reference_generation().get(), 0)));
}
