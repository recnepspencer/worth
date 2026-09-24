use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity;

#[test]
fn checkpoint_capture_includes_only_idle_ready_outputs_in_canonical_order() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let idempotency = receipt
        .idempotency_binding()
        .bind_source_partition(&[17; 32]);
    let receipt = receipt.with_idempotency_binding_for_test(idempotency);
    let occurrence = receipt.product_branch().occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let z_key = key("z-producer", 1, 1);
    let z_source = z_key.source.checkpoint_identity().bytes();
    let a_key = key("a-producer", 1, 2);
    let a_source = a_key.source.checkpoint_identity().bytes();
    let ready = |receipt| {
        DemandState::Output(WorthQueryOutputProgress::new(
            WorthQueryOutputCheckpoint::Ready(super::super::super::WorthQueryCompletedOutputDemand {
                authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
                readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
                resources: Some(crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerDemandResources::new(7, 8)),
            }),
        ))
    };
    registry.state.lock().unwrap().records.extend([
        (z_key, record(occurrence, ready(receipt.clone()), 1)),
        (a_key, record(occurrence, ready(receipt.clone()), 1)),
        (
            key("published", 1, 3),
            record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new(
                    WorthQueryOutputCheckpoint::Published {
                        receipt: receipt.clone(),
                        delivery: WorthQueryPendingOutputDelivery::NoChange,
                    },
                )),
                1,
            ),
        ),
    ]);
    let stopped_key = key("stopped", 1, 4);
    let mut stopped = WorthQueryOutputProgress::new(WorthQueryOutputCheckpoint::Ready(
        super::super::super::WorthQueryCompletedOutputDemand {
            authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
            readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
            resources: None,
        },
    ));
    stopped.stop(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        "stopped before capture",
    ));
    registry.state.lock().unwrap().records.insert(
        stopped_key,
        record(occurrence, DemandState::Output(stopped), 1),
    );

    let accepted = registry.accepted_checkpoint_identities();

    assert_eq!(accepted.len(), 2);
    assert_eq!(accepted[0].producer, "a-producer");
    assert_eq!(accepted[1].producer, "z-producer");
    assert_eq!(accepted[0].source, a_source);
    assert_eq!(accepted[1].source, z_source);
}

#[test]
fn checkpoint_output_slots_ignore_superseded_source_generations() {
    let scope = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application()
        .principal_scope()
        .scope();
    let checkpoint = |source| WorthQueryAcceptedOutputCheckpointIdentity {
        producer: "producer".to_owned(),
        source,
        scope,
        source_partition: [7; 32],
        producer_dependency: None,
        idempotency_key: source,
        resources: None,
        roles: Vec::new(),
        producer_facts: None,
    };

    assert!(checkpoint([1; 32]).same_output_slot(&checkpoint([2; 32])));
    assert!(
        !checkpoint([1; 32]).same_output_slot(&WorthQueryAcceptedOutputCheckpointIdentity {
            source_partition: [8; 32],
            ..checkpoint([2; 32])
        })
    );
}
