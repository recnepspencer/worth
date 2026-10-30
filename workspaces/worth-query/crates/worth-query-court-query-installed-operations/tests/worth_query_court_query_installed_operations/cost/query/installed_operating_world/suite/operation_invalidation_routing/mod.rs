pub(crate) use worth_query_testkit_query::facade::query::suite::operation_invalidation_routing::*;

#[test]
fn two_affected_owners_and_four_leases_report_exact_k_plus_l_routing_work() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Invalidation,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::DependencyImpact,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace(request_execution, "invalidation-indexed-k-plus-l")
        .unwrap();
    let (first_subject, first_candidate) = shared_pair(request_execution, &mut workspace);
    let (second_subject, second_candidate) = shared_pair(request_execution, &mut workspace);
    consume_empty_epoch(&mut workspace, &first_subject, &first_candidate);
    consume_empty_epoch(&mut workspace, &second_subject, &second_candidate);

    workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "two-owner-update")
        })
        .unwrap();

    let first_subject_delivery = first_subject.drain(&mut workspace).unwrap();
    let first_candidate_delivery = first_candidate.drain(&mut workspace).unwrap();
    let second_subject_delivery = second_subject.drain(&mut workspace).unwrap();
    let second_candidate_delivery = second_candidate.drain(&mut workspace).unwrap();
    assert!(first_subject_delivery.shares_invalidation_epoch_with(&first_candidate_delivery));
    assert!(second_subject_delivery.shares_invalidation_epoch_with(&second_candidate_delivery));
    assert!(!first_subject_delivery.shares_invalidation_epoch_with(&second_subject_delivery));

    let deltas = [
        first_subject
            .consumer_invalidation_delta(first_subject_delivery)
            .unwrap(),
        first_candidate
            .consumer_invalidation_delta(first_candidate_delivery)
            .unwrap(),
        second_subject
            .consumer_invalidation_delta(second_subject_delivery)
            .unwrap(),
        second_candidate
            .consumer_invalidation_delta(second_candidate_delivery)
            .unwrap(),
    ];
    assert!(deltas[0].shares_epoch_with(&deltas[1]));
    assert!(deltas[2].shares_epoch_with(&deltas[3]));
    assert!(!deltas[0].shares_epoch_with(&deltas[2]));

    let exact_owner_work = deltas[0].epoch_counters().capability_index_lookups
        + deltas[2].epoch_counters().capability_index_lookups;
    let exact_fanout =
        deltas[0].epoch_counters().fanout_targets + deltas[2].epoch_counters().fanout_targets;
    let exact_lease_work = deltas
        .iter()
        .map(|delta| delta.counters().targeted_lease_deliveries)
        .sum::<usize>();
    assert_eq!(exact_owner_work, 2, "k affected projection owners");
    assert_eq!(exact_fanout, 4, "l exact lease targets");
    assert_eq!(exact_lease_work, 4, "one delivery per exact lease");
    assert!(deltas.iter().all(|delta| {
        delta.epoch_counters().live_target_candidates_visited == 1
            && delta.epoch_counters().installed_route_index_probes == 1
    }));
    let first = deltas[0].epoch_counters();
    let second = deltas[2].epoch_counters();
    assert_eq!(
        first.live_collection_index_probes + second.live_collection_index_probes,
        1
    );
    assert_eq!(
        first.installed_collection_index_probes + second.installed_collection_index_probes,
        1
    );
    assert_eq!(
        first.installed_target_candidates_selected + second.installed_target_candidates_selected,
        2
    );
    assert_eq!(
        first.installed_candidates_skipped + second.installed_candidates_skipped,
        0
    );
    assert_eq!(
        first.target_overlap_deduplications + second.target_overlap_deduplications,
        2
    );
}
