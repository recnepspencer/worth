//! Exact fork horizons select one prior, and local absence ends inheritance.
use crate::domain_computation::primary_graph::output_lineage::retained_computation::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
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
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
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
            .create_product_branch(
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
        .lease_from_observation(child.clone())
        .unwrap();
    let nested = make(&admitted_child, "computation-nested");
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(TypeId::of::<()>(), "()");

    let entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: OutputBindingIdentity::declared("()"),
    };
    let parent = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let partition = [0x71; 32];
    let restored =
        |lineage: &mut WorthQueryApplicationOutputLineage,
         observation: &worth_runtime_world::facade::ProductBranchObservation| {
            lineage.record_restoration(lineage.binding_type(&source.output_binding), source.runtime_authority, source.schema.clone(), source.scope, observation,
            Arc::new(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence::default()),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_source_identity::RecordedSourceIdentity::Checkpoint(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity::new([0x72; 32])),
            partition, None, [0x73; 32], worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts(Arc::<[worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact]>::from([])), None, None,
            None,
        );
        };
    restored(&mut lineage, product.observation());
    let captured =
        Arc::clone(&lineage.by_source[&source][&parent.occurrence][&parent.generation][0]);
    captured.get().unwrap().mutable.lock().unwrap().computation =
        lineage.retain_computation(sealed_run_for_lineage_test());
    lineage.register_fork(product.observation(), &child);
    lineage.register_fork(admitted_child.observation(), &nested);
    let select =
        |lineage: &WorthQueryApplicationOutputLineage,
         observation: &worth_runtime_world::facade::ProductBranchObservation| {
            lineage
                .computation_cell_in_ancestry(
                    &source,
                    ProductCoordinate {
                        occurrence: observation.lifecycle_incarnation(),
                        generation: observation.reference_generation().get(),
                    },
                    partition,
                    execution,
                )
                .unwrap()
                .unwrap()
        };
    assert!(Arc::ptr_eq(&select(&lineage, &child), &captured));
    assert!(Arc::ptr_eq(&select(&lineage, &nested), &captured));
    assert!(lineage.fork_pins_computation(captured.get().unwrap()));

    lineage.release_occurrence(parent.occurrence);
    assert!(!lineage.by_source[&source].contains_key(&child.lifecycle_incarnation()));
    assert!(!lineage.by_source[&source].contains_key(&nested.lifecycle_incarnation()));
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
            local.get().unwrap().mutable.lock().unwrap().computation,
            RecordedComputation::Absent(PriorAbsence::Restored)
        ),
        "local typed absence stops ancestor reuse"
    );
    assert!(Arc::ptr_eq(&select(&lineage, &nested), &local));
    assert!(
        lineage.by_source[&source].contains_key(&parent.occurrence),
        "live descendants preserve ancestor history"
    );
    lineage.release_occurrence(child.lifecycle_incarnation());
    lineage.release_occurrence(nested.lifecycle_incarnation());
    assert!(lineage.by_source.is_empty());

    drop(local);
    drop(captured);
}
