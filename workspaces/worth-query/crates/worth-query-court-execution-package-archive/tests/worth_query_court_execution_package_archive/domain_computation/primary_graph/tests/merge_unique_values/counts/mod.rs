//! Actual equality-index calls depend on merged unique writes, not target size.
use crate::domain_computation::primary_graph::tests::merge_unique_values::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::merge_unique_values::take_lookup_limits;
#[test]
fn merge_unique_lookup_count_is_bounded_by_its_writes() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for unrelated in [0, 256] {
        let world = MergeWorld::new();
        for item in 0..unrelated {
            world.create(execution, "main", &format!("unrelated-{item}"), item);
        }
        world.fork();
        let writes = 3;
        for item in 0..writes {
            world.create(execution, FEATURE, &format!("merged-{item}"), 1000 + item);
        }
        take_lookup_limits();
        assert_eq!(world.merge(execution,), Ok(()));
        assert_eq!(take_lookup_limits(), vec![2; writes as usize]);
    }
}
