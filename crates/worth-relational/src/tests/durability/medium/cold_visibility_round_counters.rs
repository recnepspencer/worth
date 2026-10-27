//! A partially resident visibility round must abandon its instrumentation too.
use super::*;
#[test]
fn discarded_visibility_rounds_settle_only_accepted_materialization_and_leases() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for scan in [false, true] {
        let cold = recover_shape(execution, None, 4, true);
        let version = cold.runtime.current_version_id();
        // Admit one actual carrier so a later cold miss follows real work.
        cold.runtime
            .read_truth()
            .visible_entity_at_version(execution, cold.entities[0], version)
            .unwrap()
            .unwrap();
        let read = || {
            if scan {
                cold.runtime
                    .read_truth()
                    .bounded_visible_entities_of_kind(execution, KindId(1), version, 32)
                    .unwrap()
                    .unwrap()
                    .records()
                    .len()
            } else {
                cold.runtime
                    .read_truth()
                    .incoming_relations_of_kind_at_version(
                        execution,
                        cold.entities[0],
                        KindId(2),
                        version,
                    )
                    .unwrap()
                    .len()
            }
        };
        cold.runtime.performance_access().reset_counters();
        let reads = cold.source.reads.load(Ordering::Relaxed);
        let first = read();
        assert!(
            cold.source.reads.load(Ordering::Relaxed) > reads,
            "the selected cold path actually readmits another carrier"
        );
        let initial = cold.runtime.performance_access().counters();
        cold.runtime.performance_access().reset_counters();
        assert_eq!(read(), first);
        assert!(first > 0, "the producer populated the selected path");
        let warm = cold.runtime.performance_access().counters();
        if scan {
            assert!(
                warm.visible_authoritative_entity_records_materialized > 0,
                "the selected paid scan actually settles materialization instrumentation"
            );
        } else {
            assert!(
                warm.adjacency_kind_slices_leased > 0,
                "the selected traversal actually settles a substrate lease"
            );
        }
        assert_eq!(
            initial.visible_authoritative_entity_records_materialized,
            warm.visible_authoritative_entity_records_materialized,
            "discarded scan rounds have no materialization effect"
        );
        assert_eq!(
            initial.adjacency_kind_slices_leased, warm.adjacency_kind_slices_leased,
            "discarded adjacency rounds have no lease-ledger effect"
        );
    }
}
