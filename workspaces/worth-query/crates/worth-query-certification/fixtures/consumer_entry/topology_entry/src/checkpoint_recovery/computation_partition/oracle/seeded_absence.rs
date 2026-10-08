//! Runtime absence edits share the seeded alphabet with ordinary edits.
//! Policies are fixed per runtime; each boundary is exercised among every
//! other edit kind. Lifecycle interleaving belongs to slice 6.12.
use super::super::region_output::arm_own_write;
use super::differential::alphabet::{Change, Kind, Lcg, Model, KINDS};
use super::*;
use worth_query_host::facade::application_contribution::{
    discarded_computation_retention_on_this_thread_for_test as discarded_retention,
    WorthQueryPartitionedComputationFullCause as Cause, WorthQueryPartitionedComputationRun as Run,
};
use worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile;

#[derive(Clone, Copy, Debug)]
enum Boundary {
    Evict,
    Observation,
    Several,
}

fn seed(graph: &mut Graph, model: &Model, boundary: Boundary) {
    model.seed(graph);
    match boundary {
        Boundary::Evict => {}
        Boundary::Observation => {
            for number in 0..LARGEST_SET {
                let name = format!("incoming-{number}");
                facts::seed_set(graph, &name, -0.0);
                facts::seed_member(
                    graph,
                    &name,
                    &format!("oracle-entry-{}", model.heavy_number()),
                );
            }
        }
        Boundary::Several => {}
    }
}

fn profile(boundary: Boundary) -> WorthQueryOutputDemandResourceProfile {
    match boundary {
        Boundary::Evict => Default::default(),
        Boundary::Observation => WorthQueryOutputDemandResourceProfile::standard()
            .with_lineage_retained_bytes(NonZeroUsize::new(64 * 1024 * 1024).unwrap()),
        Boundary::Several => Default::default(),
    }
}

fn run<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    boundary: Boundary,
) {
    let mut rng = Lcg(0x69_ab5e_5eed);
    let mut model = if MODE == 1 {
        Model::observation(&mut rng, WORK)
    } else if matches!(boundary, Boundary::Evict) {
        Model::eviction(&mut rng, WORK)
    } else {
        Model::new(&mut rng)
    };
    let application = installation::install_variant::<REUSE, WORK, RUNS, MODE>(
        None,
        profile(boundary),
        |graph| seed(graph, &model, boundary),
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut last = demand(&request, &application).1;
    let mut history = differential::reference::Reference::new(&model);
    history.demanded(None);
    let mut command = 0x69_0000;
    let mut checked = 0;
    let mut ceilings = 0;
    for _ in 0..2 {
        let mut kinds = KINDS;
        for place in (1..kinds.len()).rev() {
            kinds.swap(place, rng.below(place + 1));
        }
        for kind in kinds.into_iter().flat_map(|kind| match kind {
            Kind::Fault => vec![Kind::Fault, Kind::Repair],
            Kind::Ceiling => vec![Kind::Ceiling, Kind::Relief],
            kind => vec![kind],
        }) {
            if kind == Kind::Evict && matches!(boundary, Boundary::Evict) && model.odd {
                model.odd = false;
                command += 1;
                history.edit(&Change::Ordinate(EVEN_Y));
                adjust(&request, &application, EVEN_Y, command);
            }
            if kind == Kind::ObservationOverBudget && MODE == 1 {
                let wide = model.wide_entry();
                history.edit(&Change::Entry(EntryEdit::delete(wide.entry)));
                history.edit(&Change::Entry(wide.clone()));
                command += 1;
                edit(
                    &request,
                    &application,
                    EntryEdit::delete(wide.entry),
                    command,
                );
                command += 1;
                edit(&request, &application, wide, command);
                let prime = demand(&request, &application).1;
                let fresh = history
                    .install::<REUSE, WORK, RUNS, MODE>(profile(boundary), |graph, model| {
                        seed(graph, model, boundary)
                    });
                let (fs, fp) = authenticate(&fresh);
                let fr = fresh.request(&fp, &fs);
                let reference = demand(&fr, &fresh).1;
                compare_prime(if prime.is_empty() { &last } else { &prime }, &reference);
                if !prime.is_empty() {
                    last = prime;
                }
                history.demanded(None);
            }
            let enlarged = if kind == Kind::Evict && matches!(boundary, Boundary::Evict) {
                // Leave the model's two-set edges inside the existing invariant
                // budget; the large even set still exceeds the typed-state ledger.
                let extra = LARGEST_SET - 16 - model.len();
                history.edit(&Change::Entry(
                    EntryEdit::create(&["even"], 1000, 1000, 1.0_f64.to_bits(), 1).batch(extra),
                ));
                command += 1;
                edit(
                    &request,
                    &application,
                    EntryEdit::create(&["even"], 1000, 1000, 1.0_f64.to_bits(), 1).batch(extra),
                    command,
                );
                // Eight retained fact rows per entry already exceed 256 KiB.
                // Only this capacity event holds the real ledger reservation;
                // later edits retain their ordinary receipt custody.
                let enlarged = application
                    .with_available_lineage_bytes_for_test(256 * 1024, || {
                        demand(&request, &application).1
                    })
                    .unwrap();
                assert!(enlarged[0].outcome.is_ok(), "eviction prime: {enlarged:?}");
                assert_eq!(enlarged.last().unwrap().published.len(), 1);
                let fresh = history
                    .install::<REUSE, WORK, RUNS, MODE>(profile(boundary), |graph, model| {
                        seed(graph, model, boundary)
                    });
                let (fs, fp) = authenticate(&fresh);
                let fr = fresh.request(&fp, &fs);
                let reference = fresh
                    .with_available_lineage_bytes_for_test(256 * 1024, || demand(&fr, &fresh).1)
                    .unwrap();
                compare_prime(&enlarged, &reference);
                history.demanded(None);
                last = enlarged;
                extra
            } else {
                0
            };
            let step = if (MODE == 1 || matches!(boundary, Boundary::Evict))
                && matches!(kind, Kind::Ceiling | Kind::Relief)
            {
                model.observation_ceiling(kind == Kind::Ceiling, WORK)
            } else {
                model.step(kind, &mut rng)
            };
            for change in step.changes {
                history.edit(&change);
                command += 1;
                match change {
                    Change::Entry(change) => edit(&request, &application, change, command),
                    Change::Ordinate(y) => adjust(&request, &application, y, command),
                }
            }
            let fresh = history
                .install::<REUSE, WORK, RUNS, MODE>(profile(boundary), |graph, model| {
                    seed(graph, model, boundary)
                });
            let (fs, fp) = authenticate(&fresh);
            let fr = fresh.request(&fp, &fs);
            if kind == Kind::Unretained {
                unretained(&application, &fresh, &model, &mut command);
            }
            arm_own_write(step.own_write);
            let (contacts, kept) = demand(&request, &application);
            arm_own_write(step.own_write);
            let reference = demand(&fr, &fresh).1;
            arm_own_write(None);
            history.demanded(step.own_write);
            if let Some(write) = step.own_write {
                model.written(write);
            }
            if enlarged > 0 {
                history.edit(&Change::Entry(EntryEdit::delete(1000).batch(enlarged)));
                command += 1;
                edit(
                    &request,
                    &application,
                    EntryEdit::delete(1000).batch(enlarged),
                    command,
                );
            }
            let target = match boundary {
                Boundary::Evict => Kind::Evict,
                Boundary::Observation => Kind::ObservationOverBudget,
                Boundary::Several => Kind::Several,
            };
            if kept.is_empty() {
                assert_ne!(kind, target, "every boundary event must execute");
                assert_eq!(contacts, 0, "a kept output initiated no execution");
                assert_eq!(
                    last.last().unwrap().outcome,
                    reference.last().unwrap().outcome,
                    "{boundary:?}, {kind:?}: kept result equals fresh computation"
                );
                assert_published_state(&last, &reference);
                continue;
            }
            assert_eq!(
                kept.iter().map(|r| &r.outcome).collect::<Vec<_>>(),
                reference.iter().map(|r| &r.outcome).collect::<Vec<_>>(),
                "{boundary:?}, {kind:?}"
            );
            assert_published_state(&kept, &reference);
            if kind == Kind::Relief && RUNS == 1 {
                assert_eq!(
                    kept[0].runs,
                    [Run::Full(Cause::Stopped)],
                    "a stopped run writes its absence before the next run"
                );
            }
            if matches!(&kept[0].outcome, Err(WorthQueryPartitionedComputationDenial::Partition {
                cause: worth_query_host::facade::application_contribution::WorthQueryComputationPartitionStop::Resource(
                    worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted), ..
            })) { ceilings += 1; }
            if kind == target {
                assert!(
                    kept[0].outcome.is_ok(),
                    "{boundary:?} boundary completes: {kept:?}"
                );
                let cause = match boundary {
                    Boundary::Evict => Cause::Evicted,
                    Boundary::Observation => Cause::ObservationOverBudget,
                    Boundary::Several => Cause::SeveralComputations,
                };
                assert_eq!(kept[0].runs, [Run::Full(cause)], "{boundary:?}: {kept:?}");
                assert_eq!(
                    kept[0].calls, reference[0].calls,
                    "full fallback enters every owner call"
                );
                checked += 1;
            }
            last = kept;
        }
    }
    assert_eq!(
        checked, 2,
        "each round's boundary event asserted its exact cause"
    );
    assert!(
        ceilings > 0,
        "{boundary:?} compared the named partition at a work stop"
    );
}

fn unretained<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    application: &Application<REUSE, WORK, RUNS, MODE>,
    fresh: &Application<REUSE, WORK, RUNS, MODE>,
    model: &Model,
    command: &mut u64,
) {
    let mut outcomes = Vec::new();
    for application in [application, fresh] {
        let (scope, principal) = authenticate(application);
        let request = application.request(&principal, &scope);
        owner::take_outcomes();
        published_states();
        discarded_retention();
        let observed = request
            .query(PlanarRead {
                body_key: SCOPE.to_owned(),
            })
            .execute()
            .unwrap();
        *command += 1;
        let committed = request
            .mutate(super::super::demand::RegionTotalsDemand {
                scope_key: SCOPE.to_owned(),
                entries: if model.odd { "odd" } else { "even" }.to_owned(),
                replacement_y: length(if model.odd { ODD_Y } else { EVEN_Y }),
            })
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(command)
            .execute_in_program::<OracleProgram<REUSE, WORK, RUNS, MODE>>(application)
            .unwrap();
        assert!(
            matches!(
                committed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "unretained mutation must commit: {committed:?}"
        );
        let outcome = owner::take_outcomes();
        assert_eq!(outcome.len(), 1);
        outcomes.push(outcome);
        // An ordinary mutation has no producer row. Its computed result is
        // configured Unretained at installation: policy suppresses retention
        // before prior delivery, and it publishes no state.
        assert!(
            published_states().is_empty(),
            "no producer row is published"
        );
        assert_eq!(discarded_retention(), [Some(Cause::RetentionPolicy)]);
    }
    // Both runtimes publish no state and discard the exact typed result,
    // while reporting the same work and named partition stop.
    assert_eq!(
        outcomes[0], outcomes[1],
        "unretained outcome, work and named stop"
    );
}

#[test]
fn seeded_eviction_steps_equal_fresh_state_and_work_boundaries() {
    let _guard = checkpoint_recovery_test_guard();
    run::<true, TOTALS_WORK, 1, 0>(Boundary::Evict);
}
#[test]
fn seeded_observation_steps_equal_fresh_state_and_work_boundaries() {
    let _guard = checkpoint_recovery_test_guard();
    run::<false, 1024, 1, 1>(Boundary::Observation);
}
#[test]
fn seeded_several_steps_equal_fresh_state_and_work_boundaries() {
    let _guard = checkpoint_recovery_test_guard();
    run::<false, TOTALS_WORK, 2, 0>(Boundary::Several);
}

/// A prime that reused its previous output still proves its retained result
/// and published state against a model-derived fresh computation.
fn compare_prime(kept: &[OracleRun], reference: &[OracleRun]) {
    assert_eq!(
        kept.last().unwrap().outcome,
        reference.last().unwrap().outcome,
        "prime outcome"
    );
    assert_published_state(kept, reference);
}
