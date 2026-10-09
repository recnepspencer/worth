//! Runtime absence edits share the seeded alphabet with ordinary edits.
//! Policies are fixed per runtime; each boundary is exercised among every
//! other edit kind, with independently derived full-build counts.
use super::super::region_output::arm_own_write;
use super::differential::alphabet::{Change, Kind, Lcg, Model, KINDS, OBSERVATION_SETS};
use super::*;
mod boundary_observation;
mod unretained;
use boundary_observation::{compare_prime, release_observation};
use unretained::unretained;
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

struct OwnWriteReset;
impl Drop for OwnWriteReset {
    fn drop(&mut self) {
        arm_own_write(None);
    }
}

mod configuration;
mod drivers;
mod eligibility;
use configuration::{profile, seed};
use eligibility::Eligibility;

fn run<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    boundary: Boundary,
    seed_number: u64,
) {
    arm_own_write(None);
    let _own_write_reset = OwnWriteReset;
    let mut rng = Lcg(seed_number);
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
    // The prime makes one decision without an own write. Later saved demands
    // carry their edit-derived decision count, never an observed vector length.
    let mut last_publications = 1;
    let mut count_model = model.clone();
    eligibility::assert_prime::<WORK, MODE>(&last, &count_model);
    let mut eligibility = Eligibility::after::<WORK, RUNS, MODE>(&count_model);
    let mut count_prior = count_model.completes_at(WORK).then(|| count_model.clone());
    let mut history = differential::reference::Reference::new(&model);
    history.demanded(None);
    let mut command = 0x69_0000;
    if MODE == 1 {
        release_observation(
            &mut model,
            &mut history,
            &request,
            &application,
            &mut command,
        );
    }
    let mut checked = 0;
    let mut ceilings = 0;
    {
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
            let mut observation_item = None;
            if kind == Kind::ObservationOverBudget && MODE == 1 {
                // Two wide items fit preparation's admitted work while their
                // shared adjacency bounds exceed comparison's ceiling.
                for change in model.observation_trim().changes {
                    history.edit(&change);
                    if let Change::Entry(change) = change {
                        command += 1;
                        edit(&request, &application, change, command);
                    }
                }
                for wide in model.wide_entries() {
                    observation_item.get_or_insert(wide.entry);
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
                }
                let executes_prime = !model.same_decision_facts(&count_model, MODE == 1);
                let prime = demand(&request, &application).1;
                assert_eq!(
                    prime.len(),
                    usize::from(executes_prime),
                    "observation prime decisions"
                );
                let fresh = history
                    .install::<REUSE, WORK, RUNS, MODE>(profile(boundary), |graph, model| {
                        seed(graph, model, boundary)
                    });
                let (fs, fp) = authenticate(&fresh);
                let fr = fresh.request(&fp, &fs);
                let reference = demand(&fr, &fresh).1;
                compare_prime(
                    if executes_prime { &prime } else { &last },
                    &reference,
                    if executes_prime { 1 } else { last_publications },
                    1,
                );
                if executes_prime {
                    count_model = model.clone();
                    for (index, run) in prime.iter().enumerate() {
                        eligibility::assert_report(
                            run,
                            eligibility.expected::<MODE, WORK>(&count_model, index),
                        );
                        assert_eq!(
                            run.calls,
                            count_model.expected_observation_calls_at(count_prior.as_ref(), WORK),
                            "observation prime"
                        );
                    }
                    eligibility = Eligibility::after::<WORK, RUNS, MODE>(&count_model);
                    count_prior = None;
                    last = prime;
                    last_publications = 1;
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
                compare_prime(&enlarged, &reference, 1, 1);
                history.demanded(None);
                count_model = model.with_eviction_entries(extra);
                for (index, run) in enlarged.iter().enumerate() {
                    eligibility::assert_report(
                        run,
                        eligibility.expected::<MODE, WORK>(&count_model, index),
                    );
                    assert_eq!(
                        run.calls,
                        count_model.expected_calls_at(count_prior.as_ref(), WORK),
                        "eviction prime"
                    );
                }
                eligibility = Eligibility::Absent(Cause::Evicted);
                count_prior = None;
                last = enlarged;
                last_publications = 1;
                extra
            } else {
                0
            };
            let step = if let Some(number) = observation_item {
                model.edit_observation_item(number, &mut rng)
            } else if (MODE == 1 || matches!(boundary, Boundary::Evict))
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
            let before_write = model.clone();
            let demanded_facts = before_write.with_eviction_entries(enlarged);
            let executes = !demanded_facts.same_decision_facts(&count_model, MODE == 1);
            let completes = if MODE == 1 {
                demanded_facts.observation_keys()
            } else {
                demanded_facts.clone()
            }
            .completes_at(WORK);
            let writes = executes
                && completes
                && step
                    .own_write
                    .is_some_and(|write| !demanded_facts.holds(write));
            let decisions = usize::from(executes) + usize::from(writes);
            let fresh_decisions = 1 + usize::from(
                completes
                    && step
                        .own_write
                        .is_some_and(|write| !demanded_facts.holds(write)),
            );
            let (contacts, kept) = demand(&request, &application);
            assert_eq!(
                (contacts, kept.len()),
                (decisions, decisions * RUNS),
                "{boundary:?}, {kind:?}: contacts and declared invocations"
            );
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
            if !executes {
                assert_ne!(kind, target, "every boundary event must execute");
                assert_eq!(contacts, 0, "a kept output initiated no execution");
                assert_eq!(
                    last.last().unwrap().outcome,
                    reference.last().unwrap().outcome,
                    "{boundary:?}, {kind:?}: kept result equals fresh computation"
                );
                compare_prime(&last, &reference, last_publications, fresh_decisions);
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
            count_model = before_write.with_eviction_entries(enlarged);
            for (decision, run) in kept.iter().enumerate() {
                eligibility::assert_report(
                    run,
                    eligibility.expected::<MODE, WORK>(&count_model, decision % RUNS),
                );
                let eligible = RUNS == 1 && kind != target;
                let prior = if eligible { count_prior.as_ref() } else { None };
                let expected = if MODE == 1 {
                    count_model.expected_observation_calls_at(prior, WORK)
                } else {
                    count_model.expected_calls_at(prior, WORK)
                };
                assert_eq!(
                    run.calls, expected,
                    "{boundary:?}, {kind:?}, decision {decision}: edit-derived calls"
                );
                eligibility = Eligibility::after::<WORK, RUNS, MODE>(&count_model);
                count_prior =
                    (RUNS == 1 && count_model.completes_at(WORK)).then(|| count_model.clone());
                if decision == 0 {
                    if let Some(write) = step.own_write {
                        count_model.written(write);
                    }
                }
            }
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
                let mut expected = if MODE == 1 {
                    model.expected_observation_calls()
                } else {
                    model.expected_calls(None)
                };
                expected.keys += enlarged;
                expected.gathers += enlarged;
                expected.kernels += enlarged;
                assert_eq!(
                    kept[0].calls, expected,
                    "full fallback enters every owner call"
                );
                checked += 1;
            }
            last = kept;
            last_publications = decisions;
            if kind == target && MODE == 1 {
                release_observation(
                    &mut model,
                    &mut history,
                    &request,
                    &application,
                    &mut command,
                );
            }
        }
    }
    assert_eq!(
        checked, 1,
        "the round's boundary event asserted its exact cause"
    );
    assert!(
        ceilings > 0,
        "{boundary:?} compared the named partition at a work stop"
    );
}
