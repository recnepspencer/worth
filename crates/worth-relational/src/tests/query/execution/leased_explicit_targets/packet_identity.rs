use super::*;
use worth_execution::ChargedBytes;

#[test]
fn last_packet_keeps_its_identity_at_width_one() {
    last_packet(1);
}
#[test]
fn last_packet_keeps_its_identity_at_width_two() {
    last_packet(2);
}
#[test]
fn last_packet_keeps_its_identity_at_width_four() {
    last_packet(4);
}

fn last_packet(width: usize) {
    const PACKETS: u64 = 5;
    const MEMORY: u64 = 128 << 10;
    // Each packet receives MEMORY / (4 * PACKETS) = 6553 result bytes.
    const CEILING: u64 = MEMORY / (4 * PACKETS);
    let runtime = runtime_with_test_schema();
    let first = create_entity_in_partition(&runtime, "a", PartitionId(7));
    let mut targets = vec![
        RecordRef::Entity(first),
        RecordRef::Entity(create_entity_in_partition(&runtime, "b", PartitionId(11))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "c", PartitionId(13))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "d", PartitionId(17))),
    ];
    let last = create_entity_in_partition(&runtime, &"e".repeat(32_000), PartitionId(19));
    targets.push(RecordRef::Entity(last));
    let snapshot = runtime.visibility_authority().snapshot();
    let plan = planned_explicit_query(&runtime, &snapshot, "packet-stop-identity", targets);
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone())
        .unwrap();
    assert_eq!(expected.complexity.packet_count, PACKETS as usize);
    assert_eq!(expected.result.entities.len(), PACKETS as usize);
    let first_record = expected
        .result
        .entities
        .iter()
        .find(|record| record.entity_id == first)
        .unwrap();
    let last_record = expected
        .result
        .entities
        .iter()
        .find(|record| record.entity_id == last)
        .unwrap();
    let first_bytes = u64::try_from(std::mem::size_of::<crate::query::data::QueryWorkerFragment>())
        .unwrap()
        + u64::try_from(std::mem::size_of_val(first_record)).unwrap()
        + first_record.additional_charged_bytes();
    // Packet zero's fragment header + one inline record + owned names fits.
    // The last packet's 32000-byte name alone exceeds 6553, at either width.
    assert_eq!(CEILING, 6553);
    assert!(
        first_bytes < CEILING,
        "packet zero fits its own result ceiling"
    );
    assert!(last_record.additional_charged_bytes() >= 32_000);
    assert!(last_record.additional_charged_bytes() > CEILING);
    let caller = lease(width, MEMORY, CancellationToken::new());
    let stopped = runtime
        .read_truth()
        .execute_query_plan_with_lease(plan, &caller);
    let Err(QueryReadExecutionStop::PacketStopped {
        boundary: Some(boundary),
        reason:
            MapStop::Failure {
                identity,
                cause: MapKernelFailure::ResultCapacityExceeded,
            },
        ..
    }) = stopped
    else {
        panic!("last packet must exceed its result ceiling at width {width}: {stopped:?}");
    };
    assert_eq!(
        boundary,
        PartitionIdentity::new(5),
        "a stopped packet must keep its identity below the packet count: width {width}"
    );
    assert_eq!(
        identity, boundary,
        "a stopped packet must keep its identity below the packet count: width {width}"
    );
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}
