//! Recovered prior correspondence is not currentness evidence until exact readmission.
use super::*;

#[test]
fn recovered_prior_correspondence_is_not_currentness_evidence_until_exact_readmission() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("the fixture's default product occurrence is live");
    let observation = product.observation();
    let runtime_authority = world.application.runtime.authority_identity().as_u64();
    let schema = world.application.installed_schema.binding_identity();
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let partition = [0x11; 32];
    let source_identity = checkpoint_identity([0x31; 32]);
    let runtime_identity = worth_query_execution::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryRuntimeSourceIdentity::new([0x99; 32]);
    let checkpoint_identity = worth_query_execution::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity::new([0x31; 32]);
    let correspondence = Arc::new(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
            TypeId::of::<RestoredOutputBinding>(),
            TypeId::of::<()>(),
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
        TypeId::of::<RestoredOutputBinding>(),
        "RestoredOutputBinding",
    );

    lineage.record_recovered_prior_output(
        TypeId::of::<RestoredOutputBinding>(),
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
    lineage.record_recovered_prior_output(
        TypeId::of::<RestoredOutputBinding>(),
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
    let semantic_source = SemanticSource {
        runtime_authority,
        schema: schema.clone(),
        scope,
        output_binding: OutputBindingIdentity::declared("RestoredOutputBinding"),
    };
    assert_eq!(
        lineage.by_source[&semantic_source][&observation.lifecycle_incarnation()].len(),
        1,
        "repeated recovery must not become a newer output"
    );

    assert!(lineage
        .qualified_output::<RestoredOutputBinding>(
            runtime_authority,
            &schema,
            scope,
            observation.lifecycle_incarnation(),
            observation.reference_generation().get(),
            runtime_identity,
            checkpoint_identity,
        )
        .is_none());

    lineage.record_restoration(
        TypeId::of::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::clone(&correspondence),
        source_identity,
        partition,
        None,
        [0x41; 32],
            worth_query_execution::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts(Arc::<[worth_query_execution::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact]>::from([])),
        None,
        None,
        None,
    );

    assert!(
        lineage
            .qualified_output::<RestoredOutputBinding>(
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

    lineage.record_restoration(
        TypeId::of::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        correspondence,
        source_identity,
        partition,
        None,
        [0x41; 32],
            worth_query_execution::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts(source_facts()),
        None,
        None,
        None,
    );

    assert!(lineage
        .qualified_output::<RestoredOutputBinding>(
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
