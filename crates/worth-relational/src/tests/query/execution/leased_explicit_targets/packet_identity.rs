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
    let targets: Vec<_> = entities.iter().copied().map(RecordRef::Entity).collect();
    let snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let plan = planned_explicit_query(&runtime, &snapshot, "packet-stop-identity", targets);
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), execution)
        .unwrap()
        .unwrap()
        .outcome;
    assert_eq!(expected.complexity.packet_count, PACKETS as usize);
    assert_eq!(expected.result.entities.len(), PACKETS as usize);
    // Immutable aspect backing now shares. Repeat a real target so the stopped
    // packet grows actual record-vector backing; every repeated row retains the
    // original record's identity, values and digest meaning.
    const REPEATS: usize = 32;
    let mut expanded = entities
        .iter()
        .copied()
        .map(RecordRef::Entity)
        .collect::<Vec<_>>();
    expanded.extend(std::iter::repeat_n(
        RecordRef::Entity(oversized),
        REPEATS - 1,
    ));
    let expanded_plan =
        planned_explicit_query(&runtime, &snapshot, "packet-stop-identity", expanded);
    let expanded_result = runtime
        .read_truth()
        .execute_query_plan(expanded_plan.clone(), execution)
        .unwrap()
        .unwrap()
        .outcome;
    assert_eq!(expanded_result.complexity.packet_count, PACKETS as usize);
    assert_eq!(
        expanded_result.result.entities.len(),
        PACKETS as usize + REPEATS - 1
    );
    for record in &expanded_result.result.entities {
        assert_eq!(
            Some(record),
            expected
                .result
                .entities
                .iter()
                .find(|original| original.entity_id == record.entity_id)
        );
    }
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
    // One record fits; repeated inline records exceed this packet's ceiling.
    assert!(
        fitting_bytes < CEILING,
        "a non-stopped packet fits its own result ceiling"
    );
    assert!(
        oversized_record
            .authoritative_aspect_state
            .as_ref()
            .unwrap()
            .owned_allocation_capacity_bytes()
            >= 32_000
    );
    assert!(oversized_record.additional_charged_bytes() < CEILING);
    assert!(REPEATS as u64 * std::mem::size_of_val(oversized_record) as u64 > CEILING);
    let caller = lease(width, MEMORY, CancellationToken::new());
    let stopped = runtime.read_truth().execute_query_plan(
        expanded_plan,
        &worth_execution::ExecutionRequest::leased(&caller),
    );
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

    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}

#[test]
fn shared_record_keeps_its_identity_through_encoding_memory_stop() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_in_partition(execution, &runtime, &"x".repeat(32_000), PartitionId(7));
    let snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "leased-oversized-record".to_owned(),
                context_id: context,
                scope: QueryScope::EntityKindScan {
                    kind_id: KindId(1),
                    partition_scope: Some(Arc::from([PartitionId(7)])),
                },
                locality: QueryLocalityClass::PartitionBounded {
                    partitions: Arc::from([PartitionId(7)]),
                },
                ordering: QueryOrderingContract::CanonicalEntityIdOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(9_176_304),
                target_count_hint: 0,
            },
        )
        .unwrap();
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), execution)
        .unwrap()
        .unwrap()
        .outcome
        .result;
    let funded = lease(1, MEMORY, CancellationToken::new());
    let actual = runtime
        .read_truth()
        .execute_query_plan(
            plan.clone(),
            &worth_execution::ExecutionRequest::leased(&funded),
        )
        .unwrap()
        .unwrap()
        .outcome
        .result;
    assert_eq!(
        actual, expected,
        "sharing preserves the original record and digest bytes"
    );
    assert!(
        actual.entities[0]
            .authoritative_aspect_state
            .as_ref()
            .unwrap()
            .owned_allocation_capacity_bytes()
            >= 32_000
    );
    // The packet shares immutable backing. Encoding the same payload still
    // allocates scratch and must preserve the original caller's real stop.
    let constrained = lease(1, 64 << 10, CancellationToken::new());
    assert!(matches!(
        runtime
            .read_truth()
            .execute_query_plan(
                plan,
                &worth_execution::ExecutionRequest::leased(&constrained)
            )
            .map(|read| read.map(|read| read.outcome)),
        Err(QueryReadExecutionStop::CompletionStopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::ResultCapacityExceeded,
                ..
            },
            ..
        })
    ));
}
