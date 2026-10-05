//! A seeded random sequence of commits, demands and checkpoint restores,
//! judged after every step.

use super::*;

mod sequence;
use sequence::{Commit, Demands, Trace, Xorshift, COMMITS, DEMANDS, HELD_ORDERS};

const SEEDS: [u64; 4] = [0x9176_3c01, 0x9176_3c02, 0x9176_3c03, 0x9170_4004];
const STEPS: usize = 40;

/// What the sequences drove, so that no case passes by never running.
#[derive(Debug, Default)]
struct Driven {
    commits: [usize; COMMITS.len()],
    held_orders: [usize; HELD_ORDERS.len()],
    free_rechecks: usize,
    /// Clean rechecks that produced a closed cached output again, after the
    /// branch retired it to admit a caller.
    evicted_rechecks: usize,
    unread_commits: usize,
    superseded_races: usize,
    restores: usize,
}

/// What a clean output costs its next demand once the branch has retired its
/// closed row for room. The row's source is read again; a root then reuses
/// its input and reaches no producer.
const REREAD: Cost = Cost {
    producer_contacts: 0,
    source_queries: 1,
};
/// A retired chain node is also decided again, by one producer contact.
const DECIDED_AGAIN: Cost = Cost {
    producer_contacts: 1,
    source_queries: 1,
};

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
fn run(seed: u64, observations: u64, driven: &mut Driven) {
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
        let court = Court::new(&application, &request, idempotency);
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
                    if let Err(stop) = raced_root.demand.advance(court.request) {
                        panic!("{at}: a funded demand stops: {stop:?}");
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
                    for index in 0..rings.len() {
                        court.demand_ring(&mut rings, index, at);
                    }
                    // Nothing committed since, so every output is clean: an
                    // open demand costs nothing, and so does a fresh one
                    // while its closed row is cached. Only a branch that
                    // admits few observations retires such a row to admit a
                    // caller, and the row's next demand then pays exactly
                    // what a retired row costs.
                    let held = held_settled!(court, HELD_ORDERS[order], a, b, c, at);
                    assert!(
                        held.iter().all(Cost::is_free) && judge_decisions(&mut rings, at) == 0,
                        "{at}: an open demand of a clean output costs nothing: {held:?}"
                    );
                    let mut retired = 0;
                    for index in 0..rings.len() {
                        let ([a, b, c, successor], decided) =
                            court.demand_ring(&mut rings, index, at);
                        let roots = [a, successor];
                        let nodes = [b, c];
                        let decided_again = nodes.iter().filter(|cost| !cost.is_free()).count();
                        assert!(
                            roots.iter().all(|cost| cost.is_free() || *cost == REREAD)
                                && nodes
                                    .iter()
                                    .all(|cost| cost.is_free() || *cost == DECIDED_AGAIN)
                                && decided == decided_again,
                            "{at}: a clean output costs nothing, or what its retired row \
                             costs: roots {roots:?}, chain nodes {nodes:?}, {decided} decisions"
                        );
                        retired +=
                            decided_again + roots.iter().filter(|cost| !cost.is_free()).count();
                    }
                    assert!(
                        retired == 0 || observations < AMPLE_OBSERVATIONS,
                        "{at}: a branch with room retires no clean output: {retired} retired"
                    );
                    driven.free_rechecks += usize::from(retired == 0);
                    driven.evicted_rechecks += usize::from(retired != 0);
                }
            }
            step += 1;
            if shape.below(8) == 0 {
                break;
            }
        }
        drop((a, b, c));
        idempotency = court.idempotency.get();
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
        run(seed, AMPLE_OBSERVATIONS, &mut driven);
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

/// The same sequences where the branch admits few product observations. No
/// caller is refused one or asked to retry: a demand, a read or a commit the
/// branch has no observation for retires closed cached outputs, and then
/// releases the sources of performed writes nobody holds, until it is
/// admitted. An open demand keeps its output, and a retired output is
/// produced again by its next demand.
#[test]
fn randomized_marking_makes_room_on_a_branch_with_few_observations() {
    let _guard = checkpoint_recovery_test_guard();
    let mut driven = Driven::default();
    for seed in SEEDS {
        run(seed, 16, &mut driven);
    }
    assert!(
        driven.evicted_rechecks > 0 && driven.free_rechecks > 0 && driven.restores > 0,
        "a cached output retired for room, an uninterrupted clean recheck and a restore are \
         driven: {driven:?}"
    );
}
