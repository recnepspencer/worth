//! A fixed native edit as one installed world's independent population grows.
use super::*;

mod lifecycle;

// Observe source-only bodies too: an unexpected producer entry there must not
// escape the affected-set assertion merely because no demand holds that body.
const OBSERVED_BODIES: [&str; 5] = ["a", "source-b", "source-c", "b", "c"];

/// Existing observations of ordinary advancement, excluding output judgments.
/// These do not measure native retained-fact visits.
#[derive(Debug, Eq, PartialEq)]
struct EditCost {
    commit_source_queries: u64,
    refresh: [Cost; 3],
    request_work: Vec<u64>,
    registry_visits: u64,
}

// The shared settlement macro judges the result after advancing. Keep these
// reads outside the native registry/request interval, just as Reading already
// keeps them outside its source-query interval.
macro_rules! observed_advance {
    ($court:expr, $demand:expr, $at:expr) => {{
        $court.application.caller_request_work_for_test();
        let before = $court.application.whole_registry_rows_visited_for_test();
        let (settlement, cost) = $demand.reading.measure(|| {
            let WorthQueryApplicationOutputDemandProgress::Settled(settlement) =
                $demand.demand.advance($court.request).unwrap()
            else {
                panic!("{}: a funded native advance must settle", $at)
            };
            let lifetime = settlement.producer_contacts_in_this_demand();
            (settlement, lifetime)
        });
        let work = $court.application.caller_request_work_for_test();
        let visits = $court.application.whole_registry_rows_visited_for_test() - before;
        $court.judge_settlement(&$demand.body, settlement.observation(), &$at);
        (cost, work, visits)
    }};
}

#[test]
fn local_edit_baseline_at_one_ten_and_hundred_copies_in_one_runtime() {
    let _guard = checkpoint_recovery_test_guard();
    // The differential observer adds full verification; measure the ordinary lane.
    let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3e00);
    let mut model = vec![Ring::seeded(0)];
    let mut open = Vec::new();
    let mut costs = Vec::new();
    Reading::decisions();
    for population in [1, 10, 100] {
        let at = format!("{population} independent rings");
        for index in model.len()..population {
            court.create_ring(index, &at);
            model.push(Ring::created(index));
        }
        for ring in &model[open.len()..] {
            let mut a = root!(court, ring.key("a"), at);
            let mut b = consumer!(court, ring.key("b"), at);
            let mut c = consumer!(court, ring.key("c"), at);
            settled!(court, a, at);
            settled!(court, b, at);
            settled!(court, c, at);
            open.push((a, b, c));
        }
        judge_decisions(&mut model, &at);
        for (a, b, c) in &mut open {
            assert!(settled!(court, c, at).is_free());
            assert!(settled!(court, b, at).is_free());
            assert!(settled!(court, a, at).is_free());
        }

        // Repeat the same 1 -> 2 edit without reinstalling the runtime.
        let roots = root_contacts(&court, &model);
        let inexact = inexact_deliveries();
        model[0].a_y = 2;
        let (_, commit_source_queries) =
            Reading::queries(|| court.write_y(&model[0].key("a"), 2, &at));
        let (a, b, c) = &mut open[0];
        let observed = [
            observed_advance!(court, c, at),
            observed_advance!(court, b, at),
            observed_advance!(court, a, at),
        ];
        let refresh = observed.each_ref().map(|(cost, _, _)| *cost);
        let request_work = observed
            .iter()
            .flat_map(|(_, work, _)| work)
            .copied()
            .collect();
        let registry_visits = observed.iter().map(|(_, _, visits)| visits).sum();
        assert_eq!(
            affected_roots(&court, &model, &roots),
            [model[0].key("a"), model[0].key("b")],
            "{at}: A changes; B republishes equal output and cuts off C"
        );
        assert_eq!(
            Reading::decisions(),
            [(model[0].key("b"), vec![model[0].root_output()])],
            "{at}: exact consumer set and current consumed value"
        );
        model[0].b_read = Some(model[0].root_output());
        assert_eq!(inexact_deliveries() - inexact, 0, "{at}: exact delivery");
        // Phase A records today's native work. Whole-map visits remain visible
        // even when the exact producer set and semantic request bill are local;
        // this baseline cannot certify the later total-locality requirement.
        costs.push(EditCost {
            commit_source_queries,
            refresh,
            request_work,
            registry_visits,
        });
        for (ring, (a, b, c)) in model.iter().zip(&mut open) {
            assert!(settled!(court, c, at).is_free());
            assert!(settled!(court, b, at).is_free());
            assert!(settled!(court, a, at).is_free());
            court.judge_chain(ring, &at);
        }
        assert!(
            Reading::decisions().is_empty(),
            "{at}: clean outputs do not decide"
        );
        assert_eq!(
            affected_roots(&court, &model, &roots),
            [model[0].key("a"), model[0].key("b")]
        );
        model[0].a_y = 1;
        court.write_y(&model[0].key("a"), 1, &at);
        let (a, b, c) = &mut open[0];
        settled!(court, c, at);
        settled!(court, b, at);
        settled!(court, a, at);
        judge_decisions(&mut model, &at);
    }
    eprintln!("ordinary local-edit baseline, populations 1/10/100: {costs:?}");
    assert!(
        costs.windows(2).all(
            |pair| pair[0].commit_source_queries == pair[1].commit_source_queries
                && pair[0].refresh == pair[1].refresh
        ),
        "equivalent edit contacts and source-query work do not grow: {costs:?}"
    );
    // Request charges include lawful history/navigation granules; allow headroom.
    let work = |cost: &EditCost| cost.request_work.iter().sum::<u64>();
    assert!(
        work(&costs[2]) <= work(&costs[0]).saturating_mul(4),
        "local request work grows with population: {costs:?}"
    );
}

fn root_contacts(court: &Court<'_, '_, '_, '_>, rings: &[Ring]) -> Vec<u64> {
    rings
        .iter()
        .flat_map(|ring| {
            OBSERVED_BODIES.map(|role| {
                let source = court
                    .request
                    .query(PlanarRead {
                        body_key: ring.key(role),
                    })
                    .execute()
                    .unwrap();
                court
                    .application
                    .producer_contacts_at_root_on_this_thread_for_test(
                        source.observed_sources()[0].root_entity_for_test(),
                    )
            })
        })
        .collect()
}

fn affected_roots(court: &Court<'_, '_, '_, '_>, rings: &[Ring], before: &[u64]) -> Vec<String> {
    let after = root_contacts(court, rings);
    rings
        .iter()
        .flat_map(|ring| OBSERVED_BODIES.map(|role| ring.key(role)))
        .zip(before.iter().zip(after))
        .filter_map(|(key, (before, after))| (after > *before).then_some(key))
        .collect()
}
