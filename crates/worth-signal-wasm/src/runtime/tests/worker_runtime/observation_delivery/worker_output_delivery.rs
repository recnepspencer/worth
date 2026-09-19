use crate::runtime::tests::support::*;
use crate::runtime::tests::worker_runtime::fixtures::portable_counter_graph::{
    set_counter, worker_shell_with_counter_graph,
};
use crate::runtime::worker_host::{
    WorkerOutputDeliveryRequest, WorkerPortableGraphPublication, WorkerRuntimeShell,
};

fn output_request(ids: &[&str]) -> WorkerOutputDeliveryRequest {
    WorkerOutputDeliveryRequest {
        output_ids: ids.iter().map(|id| (*id).to_owned()).collect(),
    }
}

#[test]
fn worker_output_delivery_packets_requested_public_outputs() {
    let mut worker_shell = worker_shell_with_counter_graph();
    let transaction = worker_shell
        .apply_committed_transaction(set_counter(7.0))
        .unwrap();

    // The published output is standing demand, so the commit already settled
    // it: no read is needed for the value to be current, and reading it does
    // not move the committed truth digest the delivery packet reports.
    assert_eq!(
        worker_shell.peek_value("doubleCounter").unwrap(),
        SignalValue::Number(14.0)
    );

    let packet = worker_shell
        .deliver_outputs(output_request(&["doubleCounter"]))
        .unwrap();
    let certification = worker_shell.certify_worker_output_delivery().unwrap();

    assert_eq!(packet.envelope_family, "outputDelivery");
    assert_eq!(packet.delivery_mode, "CommittedOutputDelivery");
    assert_eq!(packet.runtime_authority, "workerOwnedRuntime");
    assert_eq!(
        packet.worker_first_truth_digest,
        transaction.committed_truth_digest
    );
    assert_eq!(packet.output_delivery_packet_count, 1);
    assert_eq!(packet.output_delivery_breadth, 1);
    assert_eq!(packet.outputs[0].id, "doubleCounter");
    assert_eq!(packet.outputs[0].value, SignalValue::Number(14.0));
    assert!(packet.output_payload_byte_count >= 2);
    assert_eq!(packet.boundary_performance.bridge_envelope_count, 1);
    assert_eq!(packet.boundary_performance.submitted_item_count, 1);
    assert_eq!(certification.output_delivery_breadth, 1);
    assert_eq!(certification.packet_digest, packet.packet_digest);
    assert_digest_shape(&packet.output_digest);
    assert_digest_shape(&packet.packet_digest);
    assert_digest_shape(&certification.certification_digest);
}

#[test]
fn worker_output_delivery_rejects_empty_and_duplicate_requests() {
    let mut worker_shell = worker_shell_with_counter_graph();

    let empty_error = worker_shell
        .deliver_outputs(output_request(&[]))
        .unwrap_err();
    let duplicate_error = worker_shell
        .deliver_outputs(output_request(&["doubleCounter", "doubleCounter"]))
        .unwrap_err();

    assert!(empty_error.message.contains("at least one output id"));
    assert!(duplicate_error.message.contains("duplicate output id"));
}

#[test]
fn worker_output_publication_rejects_source_output_role_without_partial_graph() {
    let mut worker_shell = WorkerRuntimeShell::new(RuntimePolicySpec::default()).unwrap();

    let error = worker_shell
        .publish_graph(WorkerPortableGraphPublication {
            policy: RuntimePolicySpec::default(),
            sources: vec![SourceSpec {
                id: "counter".to_owned(),
                initial: SignalValue::Number(1.0),
                produces_aspects: None,
            }],
            recipes: vec![RecipeSpec {
                id: "doubleCounter".to_owned(),
                reads: vec![RecipeReadSpec::LegacyId("counter".to_owned())],
                expr: Expr::Sum {
                    args: vec![read("counter"), read("counter")],
                },
                when: None,
                identity: Some(IdentitySpec::Exact),
                produces_aspects: None,
            }],
            output_ids: vec!["counter".to_owned()],
        })
        .unwrap_err();

    assert!(error.message.contains("must name a published recipe"));
    assert!(worker_shell.read_value("counter").is_err());
    assert!(worker_shell.read_value("doubleCounter").is_err());
}

#[test]
fn worker_output_delivery_rejects_unknown_and_non_output_ids() {
    let mut worker_shell = worker_shell_with_counter_graph();

    let unknown_error = worker_shell
        .deliver_outputs(output_request(&["missingOutput"]))
        .unwrap_err();
    let source_error = worker_shell
        .deliver_outputs(output_request(&["counter"]))
        .unwrap_err();

    assert!(unknown_error.message.contains("not a published output"));
    assert!(source_error.message.contains("not a published output"));
}

#[test]
fn worker_output_delivery_certification_rejects_cleared_delivery_evidence() {
    let mut worker_shell = worker_shell_with_counter_graph();
    worker_shell
        .deliver_outputs(output_request(&["doubleCounter"]))
        .unwrap();
    worker_shell
        .apply_committed_transaction(set_counter(8.0))
        .unwrap();

    let error = worker_shell.certify_worker_output_delivery().unwrap_err();

    assert!(error.message.contains("delivery evidence"));
}
