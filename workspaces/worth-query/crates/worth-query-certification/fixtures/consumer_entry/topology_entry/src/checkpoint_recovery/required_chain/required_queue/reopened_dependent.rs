//! A dependent opened after its upstream's demand closed still advances in one
//! call: while it is open, the upstreams it consumes are required.

use super::*;

#[test]
fn a_reopened_dependent_refreshes_its_undemanded_stale_upstream_in_one_advance() {
    let _guard = checkpoint_recovery_test_guard();
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    let retained_positions = 8;
    let (application, _invalidation) =
        limited_application(4 * 4 * row, 128 * 1_024 * 1_024, retained_positions);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, c, mut d) = chain_with_unrelated!(application, request);
    let settled_custody = application.required_custody_bytes_for_test();
    // The unrelated caller runs until the chain's settled history has left
    // the retained positions, so the middle consumer can only be verified by
    // running it again over the outputs it consumed.
    for cycle in 0..retained_positions as u64 {
        change_root_input!(request, application, 2 + cycle % 2, 0x9176_3e00_u64 + cycle);
        settled_in_one_advance!(d, request, "the unrelated required demand");
    }
    // Every chain demand closes, then the root's input changes: the root's
    // output is stale and nothing demands it.
    drop((a, b, c));
    change_root_input!(request, application, 5, 0x9176_3e40_u64);
    take_decisions("anchor-b");
    let mut b = request
        .demand(ChainDemand("anchor-b".to_owned()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    settled_in_one_advance!(b, request, "the reopened middle consumer");
    assert_eq!(
        take_decisions("anchor-b"),
        [[6]],
        "the middle consumer read the root output refreshed inside its advance"
    );
    drop(b);
    assert!(
        application.required_custody_bytes_for_test() <= settled_custody,
        "the reopened consumer leaves no refreshed row in required custody"
    );
    drop(d);
}
