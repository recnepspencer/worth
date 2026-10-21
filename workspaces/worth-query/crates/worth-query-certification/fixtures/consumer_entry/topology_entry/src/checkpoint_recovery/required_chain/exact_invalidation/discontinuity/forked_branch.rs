//! Exact verification remains branch-local after a fork.
use super::*;

#[test]
fn a_forked_branch_verifies_what_its_parent_settled() {
    let _equivalence = EquivalenceForTest::arm_on_this_thread_for_test();
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<3>);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3dc0);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    for index in 0..rings.len() {
        court.demand_ring(&mut rings, index, "on the parent branch");
    }

    let branch = crate::checkpoint_recovery::support::host_call(&application, |phase| {
        application
            .branches()
            .fork(application.current_world())
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create(phase)
            .map_err(Box::new)
    })
    .unwrap();
    let branch_request = application.request(&principal, &scope).on_branch(branch);
    let forked = Court::new(&application, &branch_request, 0x9176_3de0);
    let mut forked_rings = rings.clone();
    let at = "on the fork, before it diverges";
    forked.demand_ring(&mut forked_rings, 0, at);

    let at = "on the fork, after its own edit";
    forked_rings[0].a_y = 6;
    forked.write_y(&forked_rings[0].key("a"), 6, at);
    let (_, decisions) = forked.demand_ring(&mut forked_rings, 0, at);
    assert!(decisions > 0, "{at}: the fork's consumer decides again");

    let at = "on the parent, after the fork's edit";
    for index in 0..rings.len() {
        let (costs, decisions) = court.demand_ring(&mut rings, index, at);
        assert!(
            costs.iter().all(|cost| cost.producer_contacts == 0) && decisions == 0,
            "{at}: the parent's ring {index} is untouched by the fork: {costs:?}"
        );
    }
}
