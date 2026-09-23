use super::*;

#[test]
fn repeated_restored_admission_joins_one_semantic_source_record() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("the fixture's default product occurrence is live");
    let observation = product.observation().clone();
    let occurrence = observation.lifecycle_incarnation();
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
        root(1),
    );
    let registry = WorthQueryOutputDemandRegistry::default();
    let restored = restored_output(observation, scope);

    let (first, inserted) = registry
        .admit_restored(
            key_with_identity("restored", 1, 1, 80),
            scope,
            occurrence,
            restored.clone(),
        )
        .expect("the recovered output admits its first demand");
    let (second, reinserted) = registry
        .admit_restored(
            key_with_identity("restored", 2, 1, 80),
            scope,
            occurrence,
            restored,
        )
        .expect("a later observation joins the recovered semantic source");

    assert!(inserted);
    assert!(!reinserted);
    assert_eq!(first.key, second.key);
    assert_eq!(registry.state.lock().unwrap().records.len(), 1);
    assert_eq!(registry.accepted_checkpoint_identities().len(), 1);
}

fn restored_output(
    observation: worth_runtime_world::facade::ProductBranchObservation,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
) -> super::super::super::WorthQueryRestoredAcceptedOutput {
    super::super::super::WorthQueryRestoredAcceptedOutput {
        checkpoint: super::super::super::WorthQueryAcceptedOutputCheckpointIdentity {
            producer: "restored".to_owned(),
            source: [0x55; 32],
            scope,
            source_partition: [0x66; 32],
            producer_dependency: None,
            idempotency_key: [0x77; 32],
            roles: Vec::new(),
        },
        correspondence: Arc::new(
            crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence::default(),
        ),
        observation,
        source_scope: scope,
        source_identity: crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([0x55; 32]),
        observed_source_facts: Arc::from([]),
    }
}
