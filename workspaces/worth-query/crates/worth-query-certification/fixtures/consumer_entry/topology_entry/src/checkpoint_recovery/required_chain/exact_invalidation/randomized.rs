//! A seeded random sequence of commits, demands and checkpoint restores,
//! judged after every step.

use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome as Performed,
    WorthQueryApplicationRequestMutationDenial, WorthQueryPerformedMutationExecutionDenial,
};

mod sequence;
use sequence::{Commit, Demands, Trace, Xorshift, COMMITS, DEMANDS, HELD_ORDERS};

const SEEDS: [u64; 4] = [0x9176_3c01, 0x9176_3c02, 0x9176_3c03, 0x9170_4004];
const STEPS: usize = 40;

/// The branch has no room for one more product observation now.
const OBSERVATION_REFUSED: WorthQueryOutputDemandDenialKind =
    WorthQueryOutputDemandDenialKind::ProductSelection(
        primary_graph::WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted,
    );

/// What the sequences drove, so that no case passes by never running.
#[derive(Debug, Default)]
struct Driven {
    commits: [usize; COMMITS.len()],
    held_orders: [usize; HELD_ORDERS.len()],
    free_rechecks: usize,
    unread_commits: usize,
    superseded_races: usize,
    restores: usize,
    retries: usize,
}

/// The three held demands settle in `$order`; what each cost, in that order.
macro_rules! held_settled {
    ($court:expr, $order:expr, $a:expr, $b:expr, $c:expr, $at:expr) => {
        $order.map(|role| match role {
            'a' => settled!($court, $a, $at),
            'b' => settled!($court, $b, $at),
            _ => settled!($court, $c, $at),
        })
    };
}

/// One seeded sequence over a world whose branch admits `observations`
/// product observations at once. A restore ends the world: the next one is
/// installed from its checkpoint and opens the held chain again.
fn run(
    seed: u64,
    observations: u64,
    retried: Option<WorthQueryOutputDemandDenialKind>,
    driven: &mut Driven,
) {
    let mut random = Xorshift(seed);
    // The draws that shape a step without changing what it commits.
    let mut shape = Xorshift(!seed);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    let mut history = Vec::new();
    let mut checkpoint = None;
    let mut idempotency = seed << 16;
    let mut step = 0;
    take_all_decisions();
    while step < STEPS {
        let opens = if checkpoint.is_some() {
            "restored"
        } else {
            "opening"
        };
        let application = install_observing(
            checkpoint.take(),
            ring_world::seed::<3>,
            Retained::AMPLE,
            observations,
        );
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let court = Court {
            retried,
            ..Court::new(&application, &request, idempotency)
        };
        history.push(format!("seed {seed:#x}, {opens} before step {step}"));
        let opening = history.last().unwrap().as_str();
        let trace = Trace(&history);
        // Ring 0 keeps its chain demanded throughout.
        let mut a = root!(court, rings[0].key("a"), opening);
        let mut b = consumer!(court, rings[0].key("b"), opening);
        let mut c = consumer!(court, rings[0].key("c"), opening);
        settled!(court, a, opening);
        settled!(court, b, opening);
        settled!(court, c, opening);
        for index in 0..rings.len() {
            court.demand_ring(&mut rings, index, opening);
        }
        drop(trace);
        // Whether a commit wrote to ring 0 since its held chain settled.
        let mut held_written = false;

        while step < STEPS {
            let index = random.below(3) as usize;
            let commit = COMMITS[random.below(COMMITS.len() as u64) as usize];
            let demands = DEMANDS[random.below(DEMANDS.len() as u64) as usize];
            let order = shape.below(HELD_ORDERS.len() as u64) as usize;
            history.push(format!(
                "seed {seed:#x}, step {step}: {commit:?} on ring {index}, then {demands:?} \
                 (held order {:?})",
                HELD_ORDERS[order]
            ));
            let at = history.last().unwrap().as_str();
            let _trace = Trace(&history);
            driven.commits[commit as usize] += 1;
            held_written |= index == 0 && commit != Commit::Nothing;
            if commit == Commit::RacingRootInput {
                let mut raced_root = root!(court, rings[index].key("a"), at);
                let mut raced_consumer = consumer!(court, rings[index].key("b"), at);
                if random.below(2) == 0 {
                    // The commit then lands between two advances. What the
                    // first one decides, it decides over the model before
                    // the commit.
                    match raced_root.demand.advance(court.request) {
                        Ok(_) => {}
                        Err(stop) if court.retries(&stop) => {}
                        Err(stop) => panic!("{at}: a funded demand stops: {stop:?}"),
                    }
                    judge_decisions(&mut rings, at);
                }
                commit.apply(&court, &mut rings[index], &mut random, at);
                // A demand that never settled on the source the commit
                // replaced stops superseded, and its caller demands again.
                if matches!(
                    raced_root.demand.advance(court.request),
                    Err(WorthQueryApplicationOutputDemandDenial::Superseded)
                ) {
                    raced_root = root!(court, rings[index].key("a"), at);
                    driven.superseded_races += 1;
                }
                settled!(court, raced_root, at);
                settled!(court, raced_consumer, at);
                judge_decisions(&mut rings, at);
                court.judge_middle(&rings[index], at);
            } else {
                commit.apply(&court, &mut rings[index], &mut random, at);
            }
            match demands {
                Demands::Nothing => {}
                Demands::Held => {
                    // One caller pumps the chain; the others settle on it.
                    let held = held_settled!(court, HELD_ORDERS[order], a, b, c, at);
                    driven.held_orders[order] += 1;
                    if !std::mem::take(&mut held_written) {
                        // No commit since wrote a field the held chain
                        // reads. Its advance also settles what those commits
                        // marked on other rings, so only its own producer
                        // contacts are judged here.
                        assert!(
                            held.iter().all(|cost| cost.producer_contacts == 0),
                            "{at}: commits to other rings reach no producer of the held \
                             chain: {held:?}"
                        );
                        driven.unread_commits += usize::from(commit != Commit::Nothing);
                    }
                    judge_decisions(&mut rings, at);
                    court.judge_chain(&rings[0], at);
                }
                Demands::Ring => {
                    court.demand_ring(&mut rings, index, at);
                }
                Demands::Everything => {
                    let _settled = held_settled!(court, HELD_ORDERS[order], a, b, c, at);
                    driven.held_orders[order] += 1;
                    held_written = false;
                    let refused = court.refusals.get();
                    for index in 0..rings.len() {
                        court.demand_ring(&mut rings, index, at);
                    }
                    // Nothing committed since, so every output is clean: an
                    // open demand costs nothing, and a fresh one contacts no
                    // producer and decides nothing. A refused observation
                    // retires a cached row, whose next demand produces it
                    // again: only a recheck no refusal interrupts is free.
                    let held = held_settled!(court, HELD_ORDERS[order], a, b, c, at);
                    let mut contacts = 0;
                    let mut decisions = judge_decisions(&mut rings, at);
                    for index in 0..rings.len() {
                        let (costs, decided) = court.demand_ring(&mut rings, index, at);
                        contacts += costs
                            .iter()
                            .map(|cost| cost.producer_contacts)
                            .sum::<usize>();
                        decisions += decided;
                    }
                    assert!(
                        court.refusals.get() != refused
                            || held.iter().all(Cost::is_free) && contacts == 0 && decisions == 0,
                        "{at}: demanding clean outputs reaches no producer: \
                         {contacts} contacts, {decisions} decisions, {held:?}"
                    );
                    driven.free_rechecks += usize::from(court.refusals.get() == refused);
                }
            }
            step += 1;
            if shape.below(8) == 0 {
                break;
            }
        }
        drop((a, b, c));
        idempotency = court.idempotency.get();
        driven.retries += court.refusals.get();
        drop(request);
        drop((principal, scope));
        if step < STEPS {
            // Whatever the last step left marked and unsettled is captured so.
            checkpoint = Some(application.capture_application_checkpoint().unwrap());
            driven.restores += 1;
        }
    }
}

#[test]
fn randomized_marking_agrees_with_full_verification() {
    let _guard = checkpoint_recovery_test_guard();
    let mut driven = Driven::default();
    for seed in SEEDS {
        run(seed, 64, None, &mut driven);
    }
    assert!(
        driven.commits.iter().all(|count| *count > 0)
            && driven.held_orders.iter().all(|count| *count > 0)
            && driven.free_rechecks > 0
            && driven.unread_commits > 0
            && driven.superseded_races > 0
            && driven.restores > 0,
        "every commit kind, every held order, the clean recheck, the unread commit, the \
         superseded race and the restore are driven: {driven:?}"
    );
}

/// The same sequences where the branch admits few product observations. A
/// funded demand or a read refused one is asked again and admitted: the
/// refusal released the observation a cached row kept.
#[test]
fn randomized_marking_outlives_a_branch_that_refuses_observations() {
    let _guard = checkpoint_recovery_test_guard();
    let mut driven = Driven::default();
    for seed in SEEDS {
        run(seed, 16, Some(OBSERVATION_REFUSED), &mut driven);
    }
    assert!(
        driven.retries > 0 && driven.restores > 0 && driven.free_rechecks > 0,
        "an observation is refused and asked for again, and a restore and an uninterrupted \
         clean recheck are driven: {driven:?}"
    );
}

/// A caller that only commits meets the rule a demand and a read meet. Every
/// performed write keeps the observation of its source, and the closed
/// cached root output of ring 1 keeps one more. The writes fill the branch
/// until it refuses one its observation; that refusal released the one the
/// cached output kept, so the same commit asked again is performed.
#[test]
fn a_commit_refused_an_observation_is_admitted_when_asked_again() {
    use primary_graph::WorthQueryOperationAuthorizationDenialKind as Authorization;
    let _guard = checkpoint_recovery_test_guard();
    let application = install_observing(None, ring_world::seed::<3>, Retained::AMPLE, 16);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f00);
    let at = "filling the branch";
    for index in 3..8 {
        court.create_ring(index, at);
    }
    take_all_decisions();
    // The source each write expects, read while the branch admits a read.
    let mut writes = Vec::new();
    for index in 3..8 {
        for (role, y) in [("a", 2), ("source-b", 2), ("source-c", 10)] {
            let body = ring_world::key(index, role);
            let read = court.read(|| {
                request
                    .query(PlanarRead {
                        body_key: body.clone(),
                    })
                    .execute()
            });
            writes.push((body, y, read.observed_sources()[0].clone()));
        }
    }
    // The root of ring 1 settles after each of three writes to its ring and
    // closes: the output stays cached on the observation it last read.
    for (role, y) in [("a", 2), ("source-c", 10), ("source-c", 11)] {
        court.write_y(&ring_world::key(1, role), y, at);
        let mut root = root!(court, ring_world::key(1, "a"), at);
        settled!(court, root, at);
    }
    let commit = |(body, y, expected): &(String, u64, _), idempotency: &_| {
        request
            .mutate(PlanarSourceAdjustment {
                scope_key: body.clone(),
                replacement_y: length(*y),
            })
            .expect_source(Clone::clone(expected))
            .idempotency(idempotency)
            .execute_performed::<program::ChainProgram, program::ChainRoot>(&application)
    };
    let refused = writes.iter().find_map(|write| {
        let idempotency = court.next_idempotency();
        match commit(write, &idempotency) {
            Ok(Performed::Performed(_)) => None,
            answer => Some((write, idempotency, answer.map(|_| "not performed"))),
        }
    });
    let Some((write, idempotency, answer)) = refused else {
        panic!("the writes fill the branch until it refuses one");
    };
    assert!(
        matches!(
            &answer,
            Err(WorthQueryPerformedMutationExecutionDenial::Mutation(
                WorthQueryApplicationRequestMutationDenial::Idempotency(denial),
            )) if denial.authorization().map(|refused| refused.kind())
                == Some(Authorization::ProductSecurityBasis(
                    primary_graph::WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted,
                ))
        ),
        "the branch refuses the commit of {} a product observation: {answer:?}",
        write.0
    );
    let again = commit(write, &idempotency);
    assert!(
        matches!(&again, Ok(Performed::Performed(_))),
        "the same commit of {} asked again is performed: {:?}",
        write.0,
        again.map(|_| "not performed")
    );
}
