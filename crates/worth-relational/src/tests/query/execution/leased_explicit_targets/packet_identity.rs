use super::*;
use worth_execution::ChargedBytes;

#[test]
fn packet_keeps_its_identity_at_width_one() {
    for stopped_ordinal in 0..9 {
        packet_stop(1, stopped_ordinal);
    }
}
#[test]
fn packet_keeps_its_identity_at_width_two() {
    for stopped_ordinal in 0..9 {
        packet_stop(2, stopped_ordinal);
    }
}
#[test]
fn packet_keeps_its_identity_at_width_four() {
    for stopped_ordinal in 0..9 {
        packet_stop(4, stopped_ordinal);
    }
}

fn packet_stop(width: usize, stopped_ordinal: usize) {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    const PACKETS: u64 = 9;
    const MEMORY: u64 = 128 << 10;
    // Each packet receives MEMORY / (4 * PACKETS) = 3640 result bytes.
    const CEILING: u64 = MEMORY / (4 * PACKETS);
    let runtime = runtime_with_test_schema();
    let partitions = [7, 11, 13, 17, 19, 23, 29, 31, 37];
    let entities: Vec<_> = partitions
        .into_iter()
        .enumerate()
        .map(|(ordinal, partition)| {
            let name = if ordinal == stopped_ordinal {
                "e".repeat(32_000)
            } else {
                "a".to_owned()
            };
            create_entity_in_partition(execution, &runtime, &name, PartitionId(partition))
        })
        .collect();
    let fitting = entities[(stopped_ordinal + 1) % entities.len()];
    let oversized = entities[stopped_ordinal];
    let targets = entities.into_iter().map(RecordRef::Entity).collect();
    let snapshot = runtime.visibility_authority().snapshot();
    let plan = planned_explicit_query(&runtime, &snapshot, "packet-stop-identity", targets);
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), execution)
        .unwrap()
        .unwrap()
        .outcome;
    assert_eq!(expected.complexity.packet_count, PACKETS as usize);
    assert_eq!(expected.result.entities.len(), PACKETS as usize);
    let fitting_record = expected
        .result
        .entities
        .iter()
        .find(|record| record.entity_id == fitting)
        .unwrap();
    let oversized_record = expected
        .result
        .entities
        .iter()
        .find(|record| record.entity_id == oversized)
        .unwrap();
    let fitting_bytes =
        u64::try_from(std::mem::size_of::<crate::query::data::QueryWorkerFragment>()).unwrap()
            + u64::try_from(std::mem::size_of_val(fitting_record)).unwrap()
            + fitting_record.additional_charged_bytes();
    // A different packet's fragment header + one inline record + owned names fits.
    // The stopped packet's 32000-byte name alone exceeds 3640, at either width.
    assert_eq!(CEILING, 3640);
    assert!(
        fitting_bytes < CEILING,
        "a non-stopped packet fits its own result ceiling"
    );
    assert!(oversized_record.additional_charged_bytes() >= 32_000);
    assert!(oversized_record.additional_charged_bytes() > CEILING);
    let caller = lease(width, MEMORY, CancellationToken::new());
    let stopped = runtime
        .read_truth()
        .execute_query_plan(plan, &worth_execution::ExecutionRequest::leased(&caller));
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
        panic!(
            "packet {stopped_ordinal} must exceed its result ceiling at width {width}: {stopped:?}"
        );
    };
    assert!(
        identity.value() <= PACKETS,
        "a packet identity is bounded by the packet count: width {width}, stopped ordinal {stopped_ordinal}"
    );
    assert_eq!(
        boundary,
        PartitionIdentity::new(u64::try_from(stopped_ordinal).unwrap() + 1),
        "a stopped packet keeps its own boundary: width {width}, stopped ordinal {stopped_ordinal}"
    );
    assert_eq!(
        identity,
        PartitionIdentity::new(u64::try_from(stopped_ordinal).unwrap() + 1),
        "a stopped packet keeps its own failure identity: width {width}, stopped ordinal {stopped_ordinal}"
    );
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}
