use crate::domain_computation::primary_graph::application_output_demand::registry::tests::recovery_posture::*;

#[test]
fn repeated_restored_admission_joins_one_semantic_source_record() {
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
    let observation = product.observation_for_test().clone();
    let occurrence = observation.lifecycle_incarnation();
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(
        root(1),
    );
    let registry = WorthQueryOutputDemandRegistry::default();
    let restored = restored_output(observation, scope);

    let (_lineage, settlement) =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let (first, inserted) = registry
        .admit_restored_for_test(
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
        .admit_restored_for_test(
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
    assert_eq!((*first.key_for_test()), (*second.key_for_test()));
    assert_eq!(
        (*(*registry.state_for_test())
            .lock()
            .unwrap()
            .records_for_test())
        .len_for_test(),
        1
    );
    assert_eq!(registry.accepted_checkpoint_identities_for_test().len(), 1);
    assert!(
        registry.posts_settlement_for_test(&settlement).unwrap(),
        "a dependent claims the restored output through its posted settlement"
    );
}

pub fn restored_output(
    observation: worth_runtime_world::facade::ProductBranchObservation,
    scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding,
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput{
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput::new_for_test(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity::new_for_test(worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint([0x11; 16], [0x22; 32]).unwrap(), ("restored".to_owned()).into(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_output_demand::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture::Performed, [0x55; 32], scope, [0x66; 32], None, [0x77; 32], (Vec::new()).into(), None, 0, None), Arc::new(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryApplicationOutputCorrespondence::default(),
        ), observation, scope, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity::new_for_test([0x55; 32]), Arc::<[_]>::from([]).into(), None)
}
