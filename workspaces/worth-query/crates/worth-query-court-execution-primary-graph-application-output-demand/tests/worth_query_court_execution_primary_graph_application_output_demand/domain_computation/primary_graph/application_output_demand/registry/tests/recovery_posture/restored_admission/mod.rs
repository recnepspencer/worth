use crate::domain_computation::primary_graph::application_output_demand::registry::tests::recovery_posture::*;

#[test]
fn repeated_restored_admission_joins_one_semantic_source_record() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("the fixture's default product occurrence is live");
    let observation = product.observation().clone();
    let occurrence = observation.lifecycle_incarnation();
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(
        root(1),
    );
    let registry = WorthQueryOutputDemandRegistry::default();
    let restored = restored_output(observation, scope);

    let (_lineage, settlement) =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let (first, inserted) = registry
        .admit_restored(
            key_with_identity("restored", 1, 1, 80),
            scope,
            occurrence,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            restored.clone(),
            &settlement,
            None,
        )
        .expect("the recovered output admits its first demand");
    let (second, reinserted) = registry
        .admit_restored(
            key_with_identity("restored", 2, 1, 80),
            scope,
            occurrence,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            restored,
            &settlement,
            None,
        )
        .expect("a later observation joins the recovered semantic source");

    assert!(inserted);
    assert!(!reinserted);
    assert_eq!(first.key, second.key);
    assert_eq!(registry.state.lock().unwrap().records.len(), 1);
    assert_eq!(registry.accepted_checkpoint_identities().len(), 1);
    assert!(
        registry.posts_settlement(&settlement).unwrap(),
        "a dependent claims the restored output through its posted settlement"
    );
}

pub fn restored_output(
    observation: worth_runtime_world::facade::ProductBranchObservation,
    scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding,
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput{
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput {
        checkpoint: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity {
            implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint([0x11; 16], [0x22; 32]).unwrap(),
            producer: ("restored".to_owned()).into(),
            posture: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture::Performed,
            source: [0x55; 32],
            scope,
            source_partition: [0x66; 32],
            producer_dependency: None,
            idempotency_key: [0x77; 32],
            roles: (Vec::new()).into(),
            producer_facts: None,
            producer_fact_wire_version: 0,
 metadata: None,
},
        correspondence: Arc::new(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence::default(),
        ),
        observation,
        source_scope: scope,
        source_identity: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity::new([0x55; 32]),
        observed_source_facts: Arc::<[_]>::from([]).into(),
        native_output_witness: None,
    }
}
