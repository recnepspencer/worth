use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_attempt::compare_and_commit::commit_receipt::certification::CertificationWorthQueryApplicationCommitReceiptForTest;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::recovery_posture::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity;

#[test]
fn checkpoint_capture_includes_only_idle_ready_outputs_in_canonical_order() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    // Accepted capture names the producing build (2296afc1a5): a Ready whose
    // receipt carries none is not reusable and is never captured.
    let implementation =
        worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap();
    let idempotency = receipt
        .idempotency_binding()
        .bind_source_partition(&[17; 32])
        .bind_producing_implementation_for_test(implementation);
    let receipt = receipt.with_idempotency_binding_for_test(idempotency);
    let occurrence = receipt.product_branch().occurrence_for_test();
    let registry = WorthQueryOutputDemandRegistry::default();
    let z_key = key("z-producer", 1, 1);
    let z_source = (*z_key.source_for_test()).checkpoint_identity().bytes_for_test();
    let a_key = key("a-producer", 1, 2);
    let a_source = (*a_key.source_for_test()).checkpoint_identity().bytes_for_test();
    let ready = |receipt| {
        DemandState::Output(WorthQueryOutputProgress::new_for_test(
            WorthQueryOutputCheckpoint::Ready(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion::for_test(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand::new_for_test(WorthQueryAcceptedOutputAuthority::Committed(receipt), worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test()))),
        ))
    };
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).extend_for_test([
        (z_key, record(occurrence, ready(receipt.clone()), 1)),
        (a_key, record(occurrence, ready(receipt.clone()), 1)),
        (
            key("published", 1, 3),
            record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new_for_test(
                    WorthQueryOutputCheckpoint::Published {
                        receipt: receipt.clone(),
                        delivery: WorthQueryPendingOutputDelivery::NoChange,
                        ready_backing: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking::for_test_for_test(),
                    },
                )),
                1,
            ),
        ),
    ]);
    let stopped_key = key("stopped", 1, 4);
    let mut stopped = WorthQueryOutputProgress::new_for_test(WorthQueryOutputCheckpoint::Ready(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion::for_test(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand::new_for_test(WorthQueryAcceptedOutputAuthority::Committed(receipt), worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test())),
    ));
    stopped.stop_for_test(WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::Superseded,
        "stopped before capture",
    ));
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).insert_for_test(
        stopped_key,
        record(occurrence, DemandState::Output(stopped), 1),
    );

    let accepted = registry.accepted_checkpoint_identities_for_test();

    assert_eq!(accepted.len(), 2);
    assert_eq!((*accepted[0].producer_for_test()).as_str(), "a-producer");
    assert_eq!((*accepted[1].producer_for_test()).as_str(), "z-producer");
    assert_eq!((*accepted[0].source_for_test()), a_source);
    assert_eq!((*accepted[1].source_for_test()), z_source);
    assert!(accepted
        .iter()
        .all(|identity| (*identity.implementation_for_test()) == implementation));
}

#[test]
fn checkpoint_output_slots_ignore_superseded_source_generations() {
    let scope = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application()
        .principal_scope()
        .scope();
    let checkpoint = |source| {
        WorthQueryAcceptedOutputCheckpointIdentity::new_for_test(worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint(
                [0x11; 16], [0x22; 32],
            )
            .unwrap(), ("producer".to_owned()).into(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture::Performed, source, scope, [7; 32], None, source, (Vec::new()).into(), None, 0, None)
    };

    assert!(checkpoint([1; 32]).same_output_slot(&checkpoint([2; 32])));
    assert!(
        !checkpoint([1; 32]).same_output_slot(&{ let __source_partition = [8; 32]; let mut __base = checkpoint([2; 32]); *__base.source_partition_mut_for_test() = __source_partition; __base })
    );
}
