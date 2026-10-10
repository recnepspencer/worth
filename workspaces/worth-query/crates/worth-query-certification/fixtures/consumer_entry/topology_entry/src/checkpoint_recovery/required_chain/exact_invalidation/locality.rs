//! One fixed edit costs the same in a world of one ring and of a hundred.

use super::*;

/// What the fixed edit and the advances after it cost.
#[derive(Debug, Eq, PartialEq)]
struct EditCost {
    /// Source queries the committing request ran.
    commit_source_queries: u64,
    /// The edited ring's last consumer, middle consumer and root, in the
    /// order they advance.
    refresh: [Cost; 3],
    decisions: usize,
    inexact_deliveries: u64,
}

/// A world of `rings` independent rings keeps every chain demanded. The first
/// ring then takes one root-input edit, and its three demands advance.
fn edit_cost(rings: usize) -> EditCost {
    let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3e00);
    let at = format!("{rings} rings");
    let mut model = vec![Ring::seeded(0)];
    for index in 1..rings {
        court.create_ring(index, &at);
        model.push(Ring::created(index));
    }
    Reading::decisions();
    let mut open = Vec::new();
    for ring in &model {
        let at = format!("{rings} rings, opening ring {}", ring.index);
        let mut a = root!(court, ring.key("a"), at);
        let mut b = consumer!(court, ring.key("b"), at);
        let mut c = consumer!(court, ring.key("c"), at);
        settled!(court, a, at);
        settled!(court, b, at);
        settled!(court, c, at);
        open.push((a, b, c));
    }
    judge_decisions(&mut model, &at);

    let inexact = inexact_deliveries();
    model[0].a_y = 2;
    let (_, commit_source_queries) = Reading::queries(|| court.write_y(&model[0].key("a"), 2, &at));
    let (a, b, c) = &mut open[0];
    let refresh = [
        settled!(court, c, at),
        settled!(court, b, at),
        settled!(court, a, at),
    ];
    let decisions = judge_decisions(&mut model, &at);

    // Every output is clean again: no copy costs a source query, a producer
    // contact or a decision, the edited one included.
    for (ring, (a, b, c)) in model.iter().zip(&mut open) {
        let clean = [
            settled!(court, c, at),
            settled!(court, b, at),
            settled!(court, a, at),
        ];
        assert!(
            clean.iter().all(Cost::is_free),
            "{at}: demanding ring {}'s clean outputs costs nothing: {clean:?}",
            ring.index
        );
        court.judge_chain(ring, &at);
    }
    assert_eq!(
        judge_decisions(&mut model, &at),
        0,
        "{at}: a clean output decides nothing again"
    );
    EditCost {
        commit_source_queries,
        refresh,
        decisions,
        inexact_deliveries: inexact_deliveries() - inexact,
    }
}

#[test]
fn a_one_field_edit_costs_the_same_at_one_and_a_hundred_copies() {
    let _guard = checkpoint_recovery_test_guard();
    let one = edit_cost(1);
    let hundred = edit_cost(100);
    assert_eq!(
        one, hundred,
        "producer contacts, source-query runs, advances and decisions do not grow with copies"
    );
    assert_eq!(
        (
            one.refresh.map(|cost| cost.producer_contacts),
            one.decisions
        ),
        ([1, 0, 0], 2),
        "the first advance refreshes the edited chain, and both its consumers decide"
    );
    assert_eq!(one.inexact_deliveries, 0, "the edit is delivered exactly");
}
