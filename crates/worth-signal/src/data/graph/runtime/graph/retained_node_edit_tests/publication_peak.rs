use super::*;

fn grow_warm(warm: &mut NodeWarmData) {
    warm.dirty_partition_scope_payload.push((
        crate::data::aspect::Aspect::new(0),
        crate::data::output::PartitionSubscription::whole_partition("warm".repeat(40_000)),
    ));
}

fn transfer_payload(payloads: &mut [RetainedNodePayload]) -> u32 {
    payloads[0].hot.state = NodeState::Clean;
    grow_warm(&mut payloads[0].warm);
    payloads[0].cold = None;
    17
}

#[test]
fn growing_warm_before_shrinking_cold_requires_the_intermediate_capacity() {
    // This scenario isolates the byte-capacity peak. The separate clone-work
    // scenario owns denial at a short attempt allowance.
    const CAPACITY_SCENARIO_WORK: usize = 500_000;
    let mut arena = prepared_arena();
    arena.cold[0] = Some(Box::new(NodeColdData {
        causality: Some(crate::data::trace::CausalityMetadata {
            kind: "cold".repeat(20_000),
            fields: Default::default(),
        }),
        ..Default::default()
    }));
    arena
        .cold
        .prepare_retained_charge(&mut Work::new(1_000))
        .unwrap();
    let original = arena.clone();
    // Independently materialize the actual first two writes while keeping the
    // old cold lane, then the final tuple after the cold replacement.
    let mut intermediate = arena.clone();
    intermediate.hot[0].as_mut().unwrap().state = NodeState::Clean;
    grow_warm(&mut intermediate.warm[0]);
    let peak = representation_charge(&intermediate);
    let mut final_state = intermediate.clone();
    final_state.cold[0] = None;
    let final_charge = representation_charge(&final_state);
    assert!(representation_charge(&arena) < final_charge);
    assert!(final_charge < peak);
    let rejected = arena
        .prepare_retained_node_edits(
            &test_ledger(),
            &[0],
            final_charge,
            &mut Work::new(CAPACITY_SCENARIO_WORK),
            transfer_payload,
        )
        .unwrap();
    assert!(matches!(rejected, RetainedNodeEditPreparation::Rejected {
        output: 17,
        denial: RetainedNodeEditDenial::CapacityExhausted { maximum, required },
    } if maximum == final_charge && required == peak));
    assert!(arena.hot.shares_storage_with(&original.hot));
    assert!(arena.warm.shares_storage_with(&original.warm));
    assert!(arena.cold.shares_storage_with(&original.cold));
    let RetainedNodeEditPreparation::Ready(prepared) = arena
        .prepare_retained_node_edits(
            &test_ledger(),
            &[0],
            peak,
            &mut Work::new(CAPACITY_SCENARIO_WORK),
            transfer_payload,
        )
        .unwrap()
    else {
        panic!("the actual publication peak must admit");
    };
    assert!(matches!(
        prepared.install(&mut arena),
        RetainedNodeEditOutcome::Installed { .. }
    ));
    assert_eq!(arena.hot, final_state.hot);
    assert_eq!(arena.warm, final_state.warm);
    assert_eq!(arena.cold, final_state.cold);
    assert_eq!(representation_charge(&arena), final_charge);
}
