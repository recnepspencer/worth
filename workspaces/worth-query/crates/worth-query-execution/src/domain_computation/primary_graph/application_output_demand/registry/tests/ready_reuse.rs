use super::*;
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryCheckpointSourceIdentity,
    WorthQueryApplicationOutputCorrespondence, WorthQueryOutputReadinessDeliveryEvidence,
};

#[test]
fn ready_ordinary_output_survives_close_and_reopens_without_scheduling() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let product_occurrence = occurrence();
    let output_key = key("producer", 5, 1);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let observation = world.selected_product().product().observation().clone();
    let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        std::any::TypeId::of::<()>(),
        Vec::new(),
        |_| None,
    )
    .expect("empty restored correspondence is valid for a registry lifetime test");
    let checkpoint = super::super::WorthQueryAcceptedOutputCheckpointIdentity {
        producer: "producer".to_owned(),
        source: [1; 32],
        scope,
        source_partition: [2; 32],
        producer_dependency: None,
        idempotency_key: [3; 32],
        resources: None,
        roles: Vec::new(),
        producer_facts: None,
    };
    let completion = super::super::WorthQueryCompletedOutputDemand {
        authority: super::super::WorthQueryAcceptedOutputAuthority::Restored(
            super::super::WorthQueryRestoredAcceptedOutput {
                checkpoint,
                correspondence: Arc::new(correspondence),
                observation,
                source_scope: scope,
                source_identity: WorthQueryCheckpointSourceIdentity::new([1; 32]),
                observed_source_facts: Arc::from([]),
            },
        ),
        readiness: WorthQueryOutputReadinessDeliveryEvidence::from_restoration(),
        resources: None,
    };
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        output_key.clone(),
        DemandRecord {
            source_scope: Some(scope),
            wake: Arc::clone(&wake),
            ..record(
                product_occurrence,
                DemandState::Output(super::super::WorthQueryOutputProgress::restored(completion)),
                1,
            )
        },
    );
    drop(interest(&registry, output_key.clone(), wake));
    let reopened = registry
        .admit(
            output_key.clone(),
            None,
            scope,
            product_occurrence,
            super::super::DemandAdmissionKind::Ordinary,
            None,
            None,
        )
        .expect("the equivalent ordinary demand reopens");
    assert!(matches!(
        registry.begin(&reopened),
        super::super::WorthQueryOutputDemandAdvanceAdmission::Ready(_)
    ));
    drop(reopened);
    registry.release_product_occurrence(product_occurrence);
    assert!(!registry
        .state
        .lock()
        .unwrap()
        .records
        .contains_key(&output_key));
}
