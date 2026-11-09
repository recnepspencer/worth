//! Public cold/warm visibility observations on the same populated ring.
use super::{current_read_basis::main_selected_version, region_source::recover_shape};
use std::sync::atomic::Ordering;
use worth_relational::facade::identity::KindId;
#[test]
fn partially_cold_visibility_reads_preserve_nonempty_warm_results() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for scan in [false, true] {
        let cold = recover_shape(execution, None, 4, true);
        let version = main_selected_version(&cold.runtime);
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
        let reads = cold.source.reads.load(Ordering::Relaxed);
        let first = read();
        assert!(
            cold.source.reads.load(Ordering::Relaxed) > reads,
            "the selected cold path actually readmits another carrier"
        );
        assert_eq!(read(), first);
        assert!(first > 0, "the producer populated the selected path");
    }
}
