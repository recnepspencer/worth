//! A demand at a retained observation older than the head. Commits after its
//! observation do not apply to it: it settles on the output current at its
//! own snapshot, and stops `Superseded` only when that output is gone.

use super::*;

macro_rules! older_root {
    ($court:expr, $observation:expr, $body:expr) => {{
        let body: String = $body;
        $court
            .request
            .at($observation)
            .demand(PlanarOutputDemand::new(body.clone()))
            .start_in_program::<program::ChainProgram, program::ChainRoot>($court.application)
            .map(|demand| Open {
                body,
                demand,
                reading: Reading::default(),
            })
    }};
}

macro_rules! older_consumer {
    ($court:expr, $observation:expr, $body:expr) => {{
        let body: String = $body;
        $court
            .request
            .at($observation)
            .demand(ChainDemand(body.clone()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                $court.application,
            )
            .map(|demand| Open {
                body,
                demand,
                reading: Reading::default(),
            })
    }};
}

/// One advance settles the started demand `$demand` of `$body` on
/// `$expected`, the output its own observation reads. It reaches no producer
/// and runs no source query.
macro_rules! settles_on {
    ($court:expr, $demand:expr, $body:expr, $expected:expr, $at:expr) => {{
        let body: String = $body;
        let (settlement, cost) = $demand.reading.measure(|| {
            let settlement = match $demand.demand.advance($court.request) {
                Ok(WorthQueryApplicationOutputDemandProgress::Settled(settled)) => settled,
                answer => panic!(
                    "{}: one advance settles the older demand of {body}; it answers {:?}",
                    $at, answer.map(|_| "Pending")
                ),
            };
            let lifetime = settlement.producer_contacts_in_this_demand();
            (settlement, lifetime)
        });
        let reads = $court
            .request
            .at(settlement.observation())
            .query(PlanarOutputRead {
                body_key: body.clone(),
            })
            .execute()
            .expect("the branch admits a courtroom read");
        assert_eq!(
            (cost.is_free(), PositiveLength::get(&reads.rows()[0].value)),
            (true, $expected),
            "{}: the older demand of {body} settles free on the output its observation reads: {cost:?}",
            $at
        );
    }};
}

/// Starts and settles the root and both consumers of `ring` at `observation`
/// on the lengths `ring` models.
fn settle_at(
    court: &Court<'_, '_, '_, '_>,
    observation: &worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation,
    ring: &Ring,
    at: &str,
) {
    let started =
        |body: &str| format!("{at}: the demand of {body} starts at the older observation");
    let key = ring.key("a");
    let mut a = older_root!(court, observation, key.clone()).unwrap_or_else(|denial| {
        panic!("{}: {denial:?}", started(&key));
    });
    settles_on!(court, a, key, ring.root_output(), at);
    for (role, expected) in [("b", ring.b_length), ("c", ring.c_length)] {
        let key = ring.key(role);
        let mut consumer =
            older_consumer!(court, observation, key.clone()).unwrap_or_else(|denial| {
                panic!("{}: {denial:?}", started(&key));
            });
        settles_on!(court, consumer, key.clone(), expected, at);
        // A settled demand answers the same settlement again.
        settles_on!(court, consumer, key, expected, at);
    }
}

/// A demand at a retained observation settles on the output current there,
/// before and after a later commit changed its input and before and after
/// the head settled on that commit. It reaches no producer and decides
/// nothing, and a head demand settles on the new output.
#[test]
fn a_demand_at_an_older_observation_ignores_later_commits() {
    let _equivalence =
        primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<3>);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f40);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    for index in 0..rings.len() {
        court.demand_ring(&mut rings, index, "before the edit");
    }
    let retained =
        crate::checkpoint_recovery::support::retain_read(court.application, court.request).unwrap();
    let older = rings.clone();

    let at = "while the retained observation is the head";
    let mut current = older_root!(court, &retained, rings[0].key("a")).unwrap();
    let cost = settled!(court, current, at);
    assert!(
        cost.is_free() && judge_decisions(&mut rings, at) == 0,
        "{at}: the retained demand settles on the clean output: {cost:?}"
    );
    drop(current);

    // Ring 0 takes the edit; ring 1 does not.
    let at = "after an edit moved the head";
    rings[0].a_y = 6;
    court.write_y(&rings[0].key("a"), 6, at);
    assert_ne!(older[0].root_output(), rings[0].root_output());
    for ring in &older[..2] {
        settle_at(&court, &retained, ring, at);
    }
    assert_eq!(
        Reading::decisions().len(),
        0,
        "{at}: a demand at an older observation decides nothing"
    );
    // The older demands left nothing behind: at the head, the edited ring
    // decides over the model and the others cost nothing.
    let ([a, ..], decisions) = court.demand_ring(&mut rings, 0, at);
    assert!(
        a.producer_contacts == 1 && decisions > 0,
        "{at}: the edited root executes again at the head: {a:?}"
    );
    for index in 1..rings.len() {
        let (costs, decisions) = court.demand_ring(&mut rings, index, at);
        assert!(
            costs.iter().all(Cost::is_free) && decisions == 0,
            "{at}: ring {index} was not edited: {costs:?}"
        );
    }

    let at = "after the head settled on the edit";
    for ring in &older[..2] {
        settle_at(&court, &retained, ring, at);
    }
    assert_eq!(
        Reading::decisions().len(),
        0,
        "{at}: a demand at an older observation decides nothing"
    );
}
