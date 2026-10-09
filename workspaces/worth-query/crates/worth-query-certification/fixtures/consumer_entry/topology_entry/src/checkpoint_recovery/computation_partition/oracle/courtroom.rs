//! Every edit is judged by fresh execution and independent model call laws.
use super::differential::alphabet::{Kind, Model};
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as Cause, WorthQueryPartitionedComputationRun as Run,
};

pub(super) const SEEDS: [u64; 2] = [0x9176_3c0b_5eed_0001, 0x9176_3c0b_5eed_0002];
const ROUNDS: usize = 1;
mod adoption;
mod bits;
mod branches;
pub(super) mod cancellation;
mod equal_output;
mod interleaving;
mod lifecycle;
mod request_work;
mod tree;
mod undo;
mod wide;

#[test]
fn seeded_edit_counts_are_derived_from_the_model_at_every_decision() {
    let _guard = checkpoint_recovery_test_guard();
    for seed in SEEDS {
        let mut prior: Option<Model> = None;
        let mut last_facts: Option<Model> = None;
        let mut last_outcome = None;
        let mut absence = Cause::FirstRun;
        differential::seeded_sequence_with_stops(seed, ROUNDS, true, true, |at, mut demanded| {
            let fresh = differential::full_run(
                demanded.model,
                demanded.reference.unwrap(),
                demanded.own_write,
            );
            let executes = last_facts.as_ref() != Some(demanded.model);
            let writes = executes
                && demanded.model.completes()
                && demanded
                    .own_write
                    .is_some_and(|write| !demanded.model.holds(write));
            let decisions = usize::from(executes) + usize::from(writes);
            assert_eq!(
                (demanded.contacts, demanded.runs.len()),
                (decisions, decisions),
                "seed {seed:#x}, {at}: model decisions"
            );
            if executes {
                assert_eq!(
                    demanded.runs.iter().map(|r| &r.outcome).collect::<Vec<_>>(),
                    fresh.iter().map(|r| &r.outcome).collect::<Vec<_>>(),
                    "{at}: bits, outcomes, charges and boundary"
                );
                assert_published_state(&demanded.runs, &fresh);
            } else {
                assert_eq!(
                    last_outcome.as_ref(),
                    Some(&fresh.last().unwrap().outcome),
                    "{at}: unchanged output remains equivalent"
                );
            }
            let mut now = demanded.model.clone();
            for (index, run) in demanded.runs.iter().enumerate() {
                assert_eq!(
                    run.calls,
                    now.expected_calls(prior.as_ref()),
                    "seed {seed:#x}, {at}, decision {index}"
                );
                let reported: u128 = run
                    .tree_runs
                    .iter()
                    .map(|tree| tree.metrics().recombined_nodes)
                    .sum();
                assert_eq!(
                    reported,
                    tree::expected_nodes(&now, prior.as_ref()),
                    "{at}: model recombinations, rotations and cutoff"
                );
                let expected = match prior.as_ref() {
                    None => Run::Full(absence),
                    Some(old) if old.odd != now.odd => Run::Full(Cause::InputChanged),
                    Some(_) => Run::Incremental,
                };
                let full_causes = match expected {
                    Run::Full(cause) => vec![cause],
                    Run::Incremental => Vec::new(),
                };
                assert_eq!(
                    run.full_preparations, full_causes,
                    "{at}: preparation cause, including failed full runs"
                );
                if now.completes() {
                    assert_eq!(run.runs, [expected], "{at}: exact run cause");
                } else {
                    assert!(
                        run.runs.is_empty(),
                        "a stopped computation has no completed run report"
                    );
                }
                absence = Cause::Stopped;
                prior = now.completes().then(|| now.clone());

                if index == 0 && writes {
                    now.written(demanded.own_write.unwrap());
                }
            }
            if executes {
                last_outcome = Some(demanded.runs.pop().unwrap().outcome);
            }
            last_facts = Some(now);
        });
    }
}

mod laws;
