//! A demand at a retained observation older than the head. Commits after its
//! observation do not apply to it: it settles on the output current at its
//! own snapshot, and stops `Superseded` only when that output is gone.

use super::*;

macro_rules! older_root {
    ($court:expr, $observation:expr, $body:expr) => {
        $court
            .request
            .at($observation)
            .demand(PlanarOutputDemand::new($body))
            .start_in_program::<program::ChainProgram, program::ChainRoot>($court.application)
    };
}

macro_rules! older_consumer {
    ($court:expr, $observation:expr, $body:expr) => {
        $court
            .request
            .at($observation)
            .demand(ChainDemand($body))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                $court.application,
            )
    };
}

/// One advance settles the started demand `$demand` of `$body` on
/// `$expected`, the output its own observation reads. It reaches no producer
/// and runs no source query.
macro_rules! settles_on {
    ($court:expr, $demand:expr, $body:expr, $expected:expr, $at:expr) => {{
        let body: String = $body;
        let before = query_entries();
        let settlement = match $demand.advance($court.request) {
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(settled)) => settled,
            answer => panic!(
                "{}: one advance settles the older demand of {body}; it answers {:?}",
                $at,
                answer.map(|_| "Pending")
            ),
        };
        let cost = Cost {
            producer_contacts: settlement.producer_contacts_in_this_demand(),
            source_queries: query_entries() - before,
        };
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

fn superseded(stop: &WorthQueryApplicationOutputDemandDenial) -> bool {
    matches!(
        stop,
        WorthQueryApplicationOutputDemandDenial::Demand(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    )
}

/// A demand at a retained observation settles on the output current there,
/// before and after a later commit changed its input and before and after
/// the head settled on that commit. It reaches no producer and decides
/// nothing, and a head demand settles on the new output.
#[test]
fn a_demand_at_an_older_observation_ignores_later_commits() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f40);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    take_all_decisions();
    for index in 0..rings.len() {
        court.demand_ring(&mut rings, index, "before the edit");
    }
    let retained = court.request.retain_read().unwrap();
    let older = rings.clone();

    let at = "while the retained observation is the head";
    let mut current = Open {
        body: rings[0].key("a"),
        demand: older_root!(court, &retained, rings[0].key("a")).unwrap(),
        producer_contacts: 0,
    };
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
        take_all_decisions().len(),
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
        take_all_decisions().len(),
        0,
        "{at}: a demand at an older observation decides nothing"
    );
}

/// Output history keeps the generations the window keeps and the ones a
/// holder pins. An observation below the window whose own generation was
/// freed is handed the older pinned row in its place. Its reader verifies
/// that row at its own snapshot, where the root it consumed has changed, and
/// never settles on it: the demand stops `Superseded`. The observation whose
/// rows are pinned still settles on them.
#[test]
fn an_observation_below_the_window_never_settles_on_an_older_pinned_output() {
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
    let court = Court::new(&application, &request, 0x9176_3f80);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    take_all_decisions();
    court.demand_ring(&mut rings, 0, "before any edit");
    let first = court.request.retain_read().unwrap();
    let first_ring = rings[0].clone();

    let at = "after the first edit";
    rings[0].a_y = 4;
    court.write_y(&rings[0].key("a"), 4, at);
    // These demands hold the rows the first observation selects.
    let mut pinned_a = older_root!(court, &first, rings[0].key("a")).unwrap();
    let mut pinned_b = older_consumer!(court, &first, rings[0].key("b")).unwrap();
    settles_on!(
        court,
        pinned_a,
        rings[0].key("a"),
        first_ring.root_output(),
        at
    );
    settles_on!(court, pinned_b, rings[0].key("b"), first_ring.b_length, at);
    court.demand_ring(&mut rings, 0, at);
    let second = court.request.retain_read().unwrap();
    let second_ring = rings[0].clone();
    settle_at(&court, &second, &second_ring, at);

    // More edits of the same root than the window keeps: the rows the
    // second observation selected are freed, the pinned ones are not.
    let at = "after the window moved past both observations";
    for y in [6, 3, 5, 2] {
        rings[0].a_y = y;
        court.write_y(&rings[0].key("a"), y, at);
        court.demand_ring(&mut rings, 0, at);
    }
    take_all_decisions();
    let root = older_root!(court, &second, rings[0].key("a")).map(drop);
    let consumer = older_consumer!(court, &second, rings[0].key("b")).map(drop);
    assert!(
        [&root, &consumer]
            .iter()
            .all(|start| start.as_ref().is_err_and(superseded)),
        "{at}: the second observation is not served an older pinned output: {root:?}, {consumer:?}"
    );
    settles_on!(
        court,
        pinned_a,
        rings[0].key("a"),
        first_ring.root_output(),
        at
    );
    settles_on!(court, pinned_b, rings[0].key("b"), first_ring.b_length, at);
    let mut again = older_consumer!(court, &first, rings[0].key("b")).unwrap();
    settles_on!(court, again, rings[0].key("b"), first_ring.b_length, at);
    assert_eq!(
        take_all_decisions().len(),
        0,
        "{at}: a demand at an older observation decides nothing"
    );
    let (costs, decisions) = court.demand_ring(&mut rings, 0, at);
    assert!(
        costs.iter().all(Cost::is_free) && decisions == 0,
        "{at}: the head is untouched: {costs:?}"
    );
}
