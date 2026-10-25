use super::*;

#[test]
fn a_shared_full_load_requires_a_separate_admitted_view_for_each_root() {
    let fixture = region_with_history();
    let make_root_view = || {
        let authority = Arc::new(RelationalRegionKindAuthority::default());
        authority
            .bind([(
                fixture.cold.partition_id,
                fixture.cold.relation_slots,
                fixture.cell.clone(),
            )])
            .unwrap();
        RelationalRegionRetention::new(fixture.cell.clone(), Some(authority))
    };
    let first = make_root_view();
    let second = make_root_view();
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    first.demand_full(&execution, 1 << 24).unwrap();
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    assert!(reads > 0 && reads < 4);
    assert!(
        fixture.cell.lease_resident_full().unwrap().is_none(),
        "a raw full load is not a root-qualified serving view"
    );
    second.demand_full(&execution, 1 << 24).unwrap();
    assert_eq!(
        fixture.source.reads.load(Ordering::SeqCst),
        reads,
        "roots share the member load"
    );
    for view in [first.full_view(), second.full_view()] {
        assert_eq!(
            view.partition()
                .entity_arena
                .metadata_history_at(fixture.entity.slot_index())
                .unwrap()
                .len(),
            fixture.expected_history
        );
        assert_eq!(
            view.commitment().digest(fixture.entity.partition_id),
            fixture.expected_digest
        );
    }
}
