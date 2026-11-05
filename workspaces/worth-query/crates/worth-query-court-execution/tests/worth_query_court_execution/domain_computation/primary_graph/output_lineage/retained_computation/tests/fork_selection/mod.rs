//! Exact fork horizons select one prior, and local absence ends inheritance.
use crate::domain_computation::primary_graph::output_lineage::retained_computation::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use worth_runtime_world::facade::ProductBranchCreationIntent;
use worth_runtime_world::facade::ProductBranchCreationPlans;
use worth_runtime_world::facade::RelationalBranchCreationPlan;
use worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome;
use worth_runtime_world::facade::RuntimeWorldCancellationSource;
use worth_runtime_world::facade::SignalBranchCreationPlan;

#[test]
fn nested_forks_select_the_captured_generation_and_stop_at_local_absence() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
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
        .unwrap();
    let make = |source: &worth_query_execution::facade::domain_computation::WorthQueryProductBranchLease, name| {
        let intent = ProductBranchCreationIntent::from_source(
            name,
            ProductBranchCreationPlans::new(
                RelationalBranchCreationPlan::ReuseExact,
                SignalBranchCreationPlan::ReuseExact,
            ),
        )
        .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(child) = world
            .application
            .product_runtime()
            .create_product_branch_for_test(
                execution,
                source,
                None,
                intent,
                &RuntimeWorldCancellationSource::new().token(),
            )
            .unwrap()
        else {
            panic!("fork performs");
        };
        child
    };
    let child = make(&product, "computation-child");
    let admitted_child = world
        .application
        .product_runtime()
        .lease_from_observation_for_test(child.clone())
        .unwrap();
    let nested = make(&admitted_child, "computation-nested");
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(TypeId::of_for_test::<()>(), "()");

    let entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test("()"));
    let parent = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let partition = [0x71; 32];
    let restored =
        |lineage: &mut WorthQueryApplicationOutputLineage,
         observation: &worth_runtime_world::facade::ProductBranchObservation| {
            lineage.record_restoration_for_test(lineage.binding_type_for_test(&(*source.output_binding_for_test())), (*source.runtime_authority_for_test()), (*source.schema_for_test()).clone(), source.scope, observation,
            Arc::new(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence::default()),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_source_identity::RecordedSourceIdentity::Checkpoint(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity::new_for_test([0x72; 32])),
            partition, None, [0x73; 32], worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(Arc::<[worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact]>::from([])), None, None,
            None,
        );
        };
    restored(&mut lineage, product.observation_for_test());
    let captured = Arc::clone(
        &(*lineage.by_source_for_test())[&source][&(*parent.occurrence_for_test())]
            [&(*parent.generation_for_test())][0],
    );
    (*(*captured.get().unwrap().mutable_for_test())
        .lock()
        .unwrap()
        .computation_mut_for_test()) =
        lineage.retain_computation_for_test(sealed_run_for_lineage_test());
    lineage.register_fork_for_test(product.observation_for_test(), &child);
    lineage.register_fork_for_test(admitted_child.observation_for_test(), &nested);
    let select =
        |lineage: &WorthQueryApplicationOutputLineage,
         observation: &worth_runtime_world::facade::ProductBranchObservation| {
            lineage
                .computation_cell_in_ancestry_for_test(
                    &source,
                    ProductCoordinate::new_for_test(
                        observation.lifecycle_incarnation(),
                        observation.reference_generation().get(),
                    ),
                    partition,
                    execution,
                )
                .unwrap()
                .unwrap()
        };
    assert!(Arc::ptr_eq(&select(&lineage, &child), &captured));
    assert!(Arc::ptr_eq(&select(&lineage, &nested), &captured));
    assert!(lineage.fork_pins_computation(captured.get().unwrap()));

    lineage.release_occurrence_for_test((*parent.occurrence_for_test()));
    assert!(!(*lineage.by_source_for_test())[&source].contains_key(&child.lifecycle_incarnation()));
    assert!(!(*lineage.by_source_for_test())[&source].contains_key(&nested.lifecycle_incarnation()));
    assert!(
        Arc::ptr_eq(&select(&lineage, &child), &captured),
        "a child with no local record selects the deleted ancestor's exact cell"
    );
    assert!(
        Arc::ptr_eq(&select(&lineage, &nested), &captured),
        "nested no-local inheritance still selects the same deleted ancestor cell"
    );

    restored(&mut lineage, &child);
    let local = select(&lineage, &child);
    assert!(!Arc::ptr_eq(&local, &captured));
    assert!(
        matches!(
            (*(*local.get().unwrap().mutable_for_test())
                .lock()
                .unwrap()
                .computation_for_test()),
            RecordedComputation::Absent(PriorAbsence::Restored)
        ),
        "local typed absence stops ancestor reuse"
    );
    assert!(Arc::ptr_eq(&select(&lineage, &nested), &local));
    assert!(
        (*lineage.by_source_for_test())[&source].contains_key(&(*parent.occurrence_for_test())),
        "live descendants preserve ancestor history"
    );
    lineage.release_occurrence_for_test(child.lifecycle_incarnation());
    lineage.release_occurrence_for_test(nested.lifecycle_incarnation());
    assert!((*lineage.by_source_for_test()).is_empty());

    drop(local);
    drop(captured);
}
