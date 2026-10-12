//! Explicit certified-dispatch and matrix callers have no neighboring authority.
use super::*;
#[test]
fn lane_71_control_callers_are_exact_and_their_neighbors_are_refused() {
    let declarations=[("crates/worth-execution/","run_owned_perturbed_for_test","workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/computation/wave_dispatch.rs"),("workspaces/worth-query/crates/worth-query-execution/","place_managed_computations_on_this_thread_for_test","workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/parallel_history/matrix/runtime.rs"),("workspaces/worth-query/crates/worth-query-execution/","prepare_without_reuse_for_test","workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/parallel_history/matrix/operation.rs"),("crates/worth-execution/","run_detached_for_test","workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/request_execution/test_placement.rs")];
    for (owner, name, caller) in declarations {
        let doors = BTreeMap::from([(name.to_owned(), BTreeSet::from([owner.to_owned()]))]);
        let source = format!("fn proof(){{ runtime.{name}(); }}");
        assert!(
            references(caller, &source, &doors).is_empty(),
            "exact caller {caller}"
        );
        let denied = references(&format!("{caller}.neighbor.rs"), &source, &doors);
        assert_eq!(
            denied.len(),
            1,
            "a neighboring path has no implicit authority"
        );
        assert_eq!(denied[0].code(), DiagnosticCode::Bc7012ControlSeal);
    }
}
