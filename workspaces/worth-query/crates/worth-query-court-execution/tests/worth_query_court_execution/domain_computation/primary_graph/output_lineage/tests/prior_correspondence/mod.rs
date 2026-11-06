//! Recovered prior correspondence is not currentness evidence until exact readmission.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence_checkpoint::court_doors_for_test::OutputCorrespondenceRestorationDoorsOps;
use super::*;

#[test]
fn recovered_prior_correspondence_is_not_currentness_evidence_until_exact_readmission() {
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
        .expect("the fixture's default product occurrence is live");
    let observation = product.observation_for_test();
    let runtime_authority = (*world.application.runtime_for_test())
        .authority_identity_for_test()
        .as_u64();
    let schema = (*world.application.installed_schema_for_test()).binding_identity();
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let partition = [0x11; 32];
    let source_identity = checkpoint_identity([0x31; 32]);
    let runtime_identity = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity::new_for_test([0x99; 32]);
    let checkpoint_identity = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity::new_for_test([0x31; 32]);
    let correspondence = Arc::new(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
            TypeId::of_for_test::<RestoredOutputBinding>(),
            TypeId::of_for_test::<()>(),
            std::iter::empty(),
            &[],
            |_| None,
            &worth_execution::ExecutionRequest::serial(
                &worth_execution::SerialRequest::from_memory(
                    worth_execution::SerialMemoryBudget::new(None),
                    worth_execution::CancellationToken::new(),
                    None,
                ),
            ),
        )
        .unwrap(),
    );
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        "RestoredOutputBinding",
    );

    lineage.record_recovered_prior_output_for_test(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation.lifecycle_incarnation(),
        observation.reference_generation().get(),
        Arc::clone(&correspondence),
        source_identity,
        partition,
        None,
        [0x41; 32],
        None,
    );
    lineage.record_recovered_prior_output_for_test(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation.lifecycle_incarnation(),
        observation.reference_generation().get() + 1,
        Arc::clone(&correspondence),
        source_identity,
        partition,
        None,
        [0x41; 32],
        None,
    );
    let semantic_source = SemanticSource::new_for_test(
        runtime_authority,
        schema.clone(),
        scope,
        OutputBindingIdentity::declared_for_test("RestoredOutputBinding"),
    );
    assert_eq!(
        (*lineage.by_source_for_test())[&semantic_source][&observation.lifecycle_incarnation()]
            .len(),
        1,
        "repeated recovery must not become a newer output"
    );

    assert!(lineage
        .qualified_output_for_test::<RestoredOutputBinding>(
            runtime_authority,
            &schema,
            scope,
            observation.lifecycle_incarnation(),
            observation.reference_generation().get(),
            runtime_identity,
            checkpoint_identity,
        )
        .is_none());

    lineage.record_restoration_for_test(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::clone(&correspondence),
        source_identity,
        partition,
        None,
        [0x41; 32],
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(Arc::<[worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact]>::from([])),
        None,
        None,
        None,
    );

    assert!(
        lineage
            .qualified_output_for_test::<RestoredOutputBinding>(
                runtime_authority,
                &schema,
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                runtime_identity,
                checkpoint_identity,
            )
            .is_none(),
        "an exact restored identity without consumed facts is not currentness evidence"
    );

    lineage.record_restoration_for_test(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        correspondence,
        source_identity,
        partition,
        None,
        [0x41; 32],
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(source_facts()),
        None,
        None,
        None,
    );

    assert!(lineage
        .qualified_output_for_test::<RestoredOutputBinding>(
            runtime_authority,
            &schema,
            scope,
            observation.lifecycle_incarnation(),
            observation.reference_generation().get(),
            runtime_identity,
            checkpoint_identity,
        )
        .is_some());
}
