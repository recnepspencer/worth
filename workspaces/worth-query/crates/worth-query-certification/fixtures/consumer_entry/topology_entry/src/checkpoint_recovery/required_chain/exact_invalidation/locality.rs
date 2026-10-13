//! A fixed native edit as one installed world's independent population grows.
use super::*;
use std::collections::BTreeSet;
use worth_query_decl::facade::application_operation::ApplicationMutationBinding;
use worth_query_host::facade::primary_graph::{
    fact_verification_on_this_thread_for_test as fact_verification,
    native_marking_on_this_thread_for_test as native_marking, WorthQueryFactVerificationForTest,
    WorthQueryNativeMarkingForTest, WorthQueryNativeMarkingPrecision,
};

mod lifecycle;

// Observe source-only bodies too: an unexpected producer entry there must not
// escape the affected-set assertion merely because no demand holds that body.
const OBSERVED_BODIES: [&str; 5] = ["a", "source-b", "source-c", "b", "c"];

/// Existing observations of ordinary advancement, excluding output judgments.
#[derive(Debug)]
struct EditCost {
    commit_source_queries: u64,
    mutation_marking: Vec<WorthQueryNativeMarkingForTest>,
    mutation_verification: WorthQueryFactVerificationForTest,
    refresh: [AdvanceCost; 3],
}

#[derive(Debug)]
struct AdvanceCost {
    cost: Cost,
    request_work: Vec<u64>,
    registry_visits: u64,
    marking: Vec<WorthQueryNativeMarkingForTest>,
    verification: WorthQueryFactVerificationForTest,
}

// The shared settlement macro judges the result after advancing. Keep these
// reads outside the native registry/request interval, just as Reading already
// keeps them outside its source-query interval.
macro_rules! observed_advance {
    ($court:expr, $demand:expr, $at:expr) => {{
        $court.application.caller_request_work_for_test();
        native_marking();
        fact_verification();
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
        let marking = native_marking();
        let verification = fact_verification();
        $court.judge_settlement(&$demand.body, settlement.observation(), &$at);
        AdvanceCost {
            cost,
            request_work: work,
            registry_visits: visits,
            marking,
            verification,
        }
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
        native_marking();
        fact_verification();
        model[0].a_y = 2;
        let (_, commit_source_queries) =
            Reading::queries(|| court.write_y(&model[0].key("a"), 2, &at));
        let mutation_marking = native_marking();
        let mutation_verification = fact_verification();
        let (a, b, c) = &mut open[0];
        let observed = [
            observed_advance!(court, c, at),
            observed_advance!(court, b, at),
            observed_advance!(court, a, at),
        ];
        let mut expected_marks: BTreeSet<_> =
            [
                (
                    model[0].key("a"),
                    <PlanarMutationBinding<CheckpointSchema> as ApplicationMutationBinding<
                        CheckpointSchema,
                    >>::IDENTITY
                        .to_owned(),
                ),
                (
                    model[0].key("b"),
                    <ChainBinding<CheckpointSchema> as ApplicationMutationBinding<
                        CheckpointSchema,
                    >>::IDENTITY
                        .to_owned(),
                ),
                (
                    model[0].key("c"),
                    <ChainBinding<CheckpointSchema> as ApplicationMutationBinding<
                        CheckpointSchema,
                    >>::IDENTITY
                        .to_owned(),
                ),
            ]
            .into_iter()
            .collect();
        if population > 1 {
            // Growth is a real PlanarEdit anchored on A. Its retained output
            // settlement also reads A, distinct from A's demanded producer.
            expected_marks.insert((
                model[0].key("a"),
                <PlanarEditBinding<CheckpointSchema> as ApplicationMutationBinding<
                    CheckpointSchema,
                >>::IDENTITY
                    .to_owned(),
            ));
        }
        assert_eq!(
            marked_outputs(&mutation_marking, &roots),
            expected_marks,
            "{at}: native source delivery marks A and the consumed B/C closure"
        );
        let no_marks = BTreeSet::new();
        for (index, advance) in observed.iter().enumerate() {
            // Advancing C publishes A's changed Length and B's equal Length.
            // The later B/A advances publish no new source effects.
            let expected = if index == 0 {
                &expected_marks
            } else {
                &no_marks
            };
            assert_eq!(
                &marked_outputs(&advance.marking, &roots),
                expected,
                "{at}: exact native output publication mark set in advance {index}"
            );
        }
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
            mutation_marking,
            mutation_verification,
            refresh: observed,
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
    for (population, cost) in [1, 10, 100].into_iter().zip(&costs) {
        eprintln!(
            "{population}: mutation marking {:?}; mutation fact verification {:?}",
            cost.mutation_marking, cost.mutation_verification
        );
        for advance in &cost.refresh {
            eprintln!(
                "{population}: advance registry visits {}; marking {:?}; fact verification {:?}",
                advance.registry_visits, advance.marking, advance.verification
            );
        }
    }
    assert!(
        costs.windows(2).all(
            |pair| pair[0].commit_source_queries == pair[1].commit_source_queries
                && pair[0].refresh.each_ref().map(|run| run.cost)
                    == pair[1].refresh.each_ref().map(|run| run.cost)
        ),
        "equivalent edit contacts and source-query work do not grow: {costs:?}"
    );
    // Request charges include lawful history/navigation granules; allow headroom.
    let work = |cost: &EditCost| {
        cost.refresh
            .iter()
            .flat_map(|run| &run.request_work)
            .sum::<u64>()
    };
    assert!(
        work(&costs[2]) <= work(&costs[0]).saturating_mul(4),
        "local request work grows with population: {costs:?}"
    );
}

struct RootContact {
    key: String,
    identity: (u32, u64, u32),
    contacts: u64,
}

fn root_contacts(court: &Court<'_, '_, '_, '_>, rings: &[Ring]) -> Vec<RootContact> {
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
                let root = source.observed_sources()[0].root_entity_for_test();
                let contacts = court
                    .application
                    .producer_contacts_at_root_on_this_thread_for_test(root);
                RootContact {
                    key: ring.key(role),
                    identity: (
                        root.partition_value(),
                        root.local_slot_value(),
                        root.generation_value(),
                    ),
                    contacts,
                }
            })
        })
        .collect()
}

fn affected_roots(
    court: &Court<'_, '_, '_, '_>,
    rings: &[Ring],
    before: &[RootContact],
) -> Vec<String> {
    let after = root_contacts(court, rings);
    rings
        .iter()
        .flat_map(|ring| OBSERVED_BODIES.map(|role| ring.key(role)))
        .zip(before.iter().zip(after))
        .filter_map(|(key, (before, after))| (after.contacts > before.contacts).then_some(key))
        .collect()
}

/// Physical historical identities remain in the report. Judge their semantic
/// roots and actual installed output bindings, without pinning alias counts.
fn marked_outputs(
    reports: &[WorthQueryNativeMarkingForTest],
    roots: &[RootContact],
) -> BTreeSet<(String, String)> {
    reports
        .iter()
        .flat_map(|report| {
            assert!(
                matches!(report.precision, WorthQueryNativeMarkingPrecision::Exact(_)),
                "ample ordinary publication has exact delivery: {report:?}"
            );
            report
                .affected
                .as_ref()
                .expect("exact native delivery exposes affected outputs")
        })
        .map(|output| {
            let identity = (
                output.root.partition_id(),
                output.root.local_slot(),
                output.root.generation(),
            );
            let root = roots
                .iter()
                .find(|root| root.identity == identity)
                .unwrap_or_else(|| panic!("marking reached an unrelated root: {output:?}"));
            (root.key.clone(), output.output_binding.to_string())
        })
        .collect()
}
