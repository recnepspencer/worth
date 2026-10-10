//! Where marking has no exact answer, full verification answers: no producer
//! is reached, nothing decides again, and every output agrees with the model.

use super::*;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

/// Three rings settle, and the world is captured.
fn settled_checkpoint(
    rings: &mut [Ring],
) -> application_installation::WorthQueryApplicationCheckpoint {
    let application = install(None, ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3d00);
    for index in 0..rings.len() {
        court.demand_ring(rings, index, "before the checkpoint");
    }
    drop((principal, scope));
    application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap()
}

#[test]
fn a_restored_world_verifies_each_root_and_decides_each_consumer_once() {
    let _guard = checkpoint_recovery_test_guard();
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    let checkpoint = settled_checkpoint(&mut rings);

    let application = install(Some(checkpoint), ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3d40);
    let inexact = inexact_deliveries();
    let at = "after the restore";
    for index in 1..rings.len() {
        // Readmission compares a restored root's checkpoint facts and output
        // in full at the head, and that comparison is recorded as marks. A
        // restored output claims nothing upstream, so no consumer is
        // restored: each executes once and decides over its upstream again.
        let ([a, b, c, successor], decisions) = court.demand_ring(&mut rings, index, at);
        assert!(
            [a, successor]
                .iter()
                .all(|cost| cost.producer_contacts == 0 && cost.source_queries == 0),
            "{at}: each root of ring {index} is readmitted from its checkpoint facts and \
             reaches no producer: {a:?}, {successor:?}"
        );
        assert_eq!(
            (b.producer_contacts, c.producer_contacts, decisions),
            (1, 1, 2),
            "{at}: each consumer of ring {index} decides once over its upstream"
        );
        let (again, decisions) = court.demand_ring(&mut rings, index, at);
        assert!(
            again
                .iter()
                .all(|cost| cost.producer_contacts == 0 && cost.source_queries == 0)
                && decisions == 0,
            "{at}: demanded again, ring {index} reads its marks: {again:?}"
        );
    }

    let at = "an edit after the restore";
    rings[0].a_y = 5;
    court.write_y(&rings[0].key("a"), 5, at);
    let mut a = root!(court, rings[0].key("a"), at);
    let root = settled!(court, a, at);
    assert_eq!(
        (root.producer_contacts, court.length(&rings[0].key("a"))),
        (1, rings[0].root_output()),
        "{at}: the edited root executes again and publishes what the model computes"
    );
    Reading::decisions();
    for index in 1..rings.len() {
        let (costs, decisions) = court.demand_ring(&mut rings, index, at);
        assert!(
            costs.iter().all(|cost| cost.producer_contacts == 0) && decisions == 0,
            "{at}: ring {index} was not edited: {costs:?}"
        );
    }
    assert_eq!(
        inexact_deliveries() - inexact,
        0,
        "{at}: every commit is delivered exactly"
    );
}

#[test]
fn marks_older_than_the_retained_window_are_verified_in_full() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(
        None,
        ring_world::seed::<3>,
        Retained {
            commit_positions: 2,
            ..Retained::AMPLE
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3d80);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    let at = "inside the retained window";
    let mut a = root!(court, rings[0].key("a"), at);
    let mut b = consumer!(court, rings[0].key("b"), at);
    let mut c = consumer!(court, rings[0].key("c"), at);
    let mut far_a = root!(court, rings[2].key("a"), at);
    let mut far_b = consumer!(court, rings[2].key("b"), at);
    settled!(court, a, at);
    settled!(court, b, at);
    settled!(court, c, at);
    settled!(court, far_a, at);
    settled!(court, far_b, at);
    judge_decisions(&mut rings, at);

    // The first ring's edit is followed by more commits than the window
    // retains, all on another ring, before any open chain is demanded again.
    let at = "outside the retained window";
    rings[0].a_y = 4;
    court.write_y(&rings[0].key("a"), 4, at);
    for y in [2, 3, 4, 5] {
        rings[1].a_y = y;
        court.write_y(&rings[1].key("a"), y, at);
    }
    let refresh = [
        settled!(court, c, at),
        settled!(court, b, at),
        settled!(court, a, at),
    ];
    assert_eq!(
        refresh.map(|cost| cost.producer_contacts),
        [1, 0, 0],
        "{at}: the edited chain refreshes on its first advance: {refresh:?}"
    );
    assert!(
        judge_decisions(&mut rings, at) > 0,
        "{at}: the middle consumer decides over the new root output"
    );
    court.judge_chain(&rings[0], at);
    let far = [settled!(court, far_b, at), settled!(court, far_a, at)];
    let again = [
        settled!(court, far_b, at),
        settled!(court, far_a, at),
        settled!(court, c, at),
        settled!(court, b, at),
        settled!(court, a, at),
    ];
    // The consumer is demanded first: its one full verification covers the
    // root it consumed, so the root is already marked clean when demanded.
    assert_eq!(
        far.map(|cost| (cost.producer_contacts, cost.source_queries)),
        [(0, 1), (0, 0)],
        "{at}: the unedited chain is verified once in full and reaches no producer: {far:?}"
    );
    assert!(
        again
            .iter()
            .all(|cost| cost.producer_contacts == 0 && cost.source_queries == 0)
            && judge_decisions(&mut rings, at) == 0,
        "{at}: a verified output is marked clean again and runs no source query: {again:?}"
    );
    court.judge_middle(&rings[2], at);
    court.demand_ring(&mut rings, 1, at);
}

#[test]
fn an_output_demanded_inside_every_window_never_leaves_it() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(
        None,
        ring_world::seed::<3>,
        Retained {
            commit_positions: 4,
            ..Retained::AMPLE
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3dc0);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    let at = "before the commits";
    let mut far_a = root!(court, rings[2].key("a"), at);
    let mut far_b = consumer!(court, rings[2].key("b"), at);
    let mut far_c = consumer!(court, rings[2].key("c"), at);
    settled!(court, far_a, at);
    settled!(court, far_b, at);
    settled!(court, far_c, at);
    judge_decisions(&mut rings, at);

    // Twenty-four commits on another ring rotate the four-position window
    // six times. The untouched chain is demanded after every second one: a
    // clean demand moves what it verified to the head, so no rotation leaves
    // it behind and none costs a full verification.
    for round in 0..12_u64 {
        let at = format!("round {round}");
        for y in [2 + 2 * (round % 3), 3 + 2 * (round % 3)] {
            rings[1].a_y = y;
            court.write_y(&rings[1].key("a"), y, &at);
        }
        let clean = [
            settled!(court, far_c, at),
            settled!(court, far_b, at),
            settled!(court, far_a, at),
        ];
        assert!(
            clean.iter().all(Cost::is_free),
            "{at}: the untouched chain is still inside the window: {clean:?}"
        );
    }
    let at = "after the commits";
    assert_eq!(
        judge_decisions(&mut rings, at),
        0,
        "{at}: the untouched chain decides nothing again"
    );
    court.judge_chain(&rings[2], at);
    court.demand_ring(&mut rings, 1, at);
}

#[test]
fn a_forked_branch_verifies_what_its_parent_settled() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3dc0);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    for index in 0..rings.len() {
        court.demand_ring(&mut rings, index, "on the parent branch");
    }

    let branch = application
        .branches()
        .fork(application.current_world())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
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

/// A consumer demanded before its upstream after a restore never settles on
/// what the checkpoint held: it stops, as in a world that has not demanded
/// that upstream. Demanded after it, the chain decides over the edited root.
#[test]
fn a_restored_consumer_never_settles_over_a_stale_upstream() {
    let _guard = checkpoint_recovery_test_guard();
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    let checkpoint = settled_checkpoint(&mut rings);

    let application = install(Some(checkpoint), ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3a40);
    let at = "an edit after the restore, the consumer demanded first";
    rings[0].a_y = 5;
    court.write_y(&rings[0].key("a"), 5, at);
    let mut early = consumer!(court, rings[0].key("b"), at);
    let stop = (0..8)
        .find_map(|_| match early.demand.advance(court.request) {
            Ok(WorthQueryApplicationOutputDemandProgress::Pending) => None,
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(_)) => {
                panic!("{at}: the consumer settles while its upstream is stale")
            }
            Err(stop) => Some(stop),
        })
        .unwrap_or_else(|| panic!("{at}: the consumer neither settles nor stops"));
    assert!(
        !offers_retry(&stop) && judge_decisions(&mut rings, at) == 0,
        "{at}: its upstream has no output to consume: {stop:?}"
    );
    drop(early);

    let at = "the chain demanded in dependency order";
    let (_, decisions) = court.demand_ring(&mut rings, 0, at);
    assert_eq!(
        decisions, 2,
        "{at}: both consumers decide over the new root"
    );
}

/// A restored root's own demand is open and never advances. The chain that
/// consumed it refreshes it: after an edit, advancing only the last consumer
/// settles the whole chain over the new root.
#[test]
fn a_restored_chain_settles_through_its_last_consumer_alone() {
    let _guard = checkpoint_recovery_test_guard();
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    let checkpoint = settled_checkpoint(&mut rings);

    let application = install(Some(checkpoint), ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3a80);
    let at = "after the restore";
    let mut a = root!(court, rings[0].key("a"), at);
    let mut b = consumer!(court, rings[0].key("b"), at);
    let mut c = consumer!(court, rings[0].key("c"), at);
    settled!(court, b, at);
    settled!(court, c, at);
    assert_eq!(judge_decisions(&mut rings, at), 2);

    let at = "an edit after the restore, only the last consumer advanced";
    rings[0].a_y = 5;
    court.write_y(&rings[0].key("a"), 5, at);
    settled!(court, c, at);
    assert_eq!(
        judge_decisions(&mut rings, at),
        1,
        "{at}: B decides over the new root; its equal output leaves C unchanged"
    );
    court.judge_chain(&rings[0], at);
    let rest = [settled!(court, a, at), settled!(court, b, at)];
    assert!(
        rest.iter().all(Cost::is_free) && judge_decisions(&mut rings, at) == 0,
        "{at}: the root and the middle consumer settled on that advance: {rest:?}"
    );
}
