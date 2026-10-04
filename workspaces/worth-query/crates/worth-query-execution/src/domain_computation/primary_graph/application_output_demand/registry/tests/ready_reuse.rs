use super::*;
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryCheckpointSourceIdentity,
    WorthQueryApplicationOutputCorrespondence, WorthQueryOutputReadinessDeliveryEvidence,
};

#[test]
fn ready_output_reopens_and_only_its_exact_restored_predecessor_refreshes_it() {
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
        std::any::TypeId::of::<()>(),
        std::collections::BTreeSet::new(),
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
    let super::super::WorthQueryOutputDemandAdvanceAdmission::Ready(completion) =
        registry.begin(&reopened)
    else {
        panic!("retained output remains ready")
    };
    let mut lookalike = completion.authority.clone();
    if let super::super::WorthQueryAcceptedOutputAuthority::Restored(restored) = &mut lookalike {
        restored.checkpoint.idempotency_key = [4; 32];
    }
    let unrelated = registry
        .admit(
            output_key.clone(),
            None,
            scope,
            product_occurrence,
            super::super::DemandAdmissionKind::Ordinary,
            None,
            Some(&lookalike),
        )
        .unwrap();
    assert!(
        matches!(
            registry.begin(&unrelated),
            super::super::WorthQueryOutputDemandAdvanceAdmission::Ready(_)
        ),
        "a lookalike predecessor cannot reopen the accepted output"
    );
    drop(unrelated);
    let refreshed = registry
        .admit(
            output_key.clone(),
            None,
            scope,
            product_occurrence,
            super::super::DemandAdmissionKind::Ordinary,
            None,
            Some(&completion.authority),
        )
        .unwrap();
    assert!(matches!(
        registry.begin(&refreshed),
        super::super::WorthQueryOutputDemandAdvanceAdmission::Schedule(None)
    ));
    registry.finish_scheduling(
        &refreshed,
        None,
        &mut Ok(super::super::WorthQueryOutputSchedulingResult::Scheduled),
    );
    assert!(
        matches!(registry.begin(&refreshed), super::super::WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of: Some(identity) } if identity == [3; 32]),
        "the exact restored predecessor reopens with its retained idempotency lineage"
    );
    drop(refreshed);
    drop(reopened);
    registry.release_product_occurrence(product_occurrence);
    assert!(!registry
        .state
        .lock()
        .unwrap()
        .records
        .contains_key(&output_key));
}
