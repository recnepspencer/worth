//! The literal table is exercised through admitted edits, with shape assertions in the model.
use super::*;
use crate::checkpoint_recovery::computation_partition::region_output::arm_own_write;
#[test]
fn named_edit_laws_have_the_declared_occurrences_and_exact_calls() {
    let _guard = checkpoint_recovery_test_guard();
    for (number, case) in Model::law_cases().into_iter().enumerate() {
        let application = install(|graph| case.before.seed(graph));
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let prime = demand(&request, &application).1;
        if case.before.completes() {
            assert_eq!(prime[0].runs, [Run::Full(Cause::FirstRun)]);
        } else {
            assert!(prime[0].runs.is_empty());
        }
        let mut reference = differential::reference::Reference::new(&case.before);
        reference.demanded(None);
        for (index, change) in case.changes.iter().enumerate() {
            reference.edit(change);
            let command = 0x612_4000 + number as u64 * 16 + index as u64;
            match change {
                differential::alphabet::Change::Entry(change) => {
                    edit(&request, &application, change.clone(), command)
                }
                differential::alphabet::Change::Ordinate(y) => {
                    adjust(&request, &application, *y, command)
                }
            }
        }
        let unchanged = case.calls == [0; 4];
        let decisions = if unchanged {
            0
        } else {
            1 + usize::from(case.own_write.is_some())
        };
        arm_own_write(case.own_write);
        let (contacts, runs) = demand(&request, &application);
        arm_own_write(None);
        assert_eq!(
            (contacts, runs.len()),
            (decisions, decisions),
            "{}",
            case.name
        );
        let fresh = differential::full_run(&case.after, &reference, case.own_write);
        let mut now = case.after.clone();
        for (index, run) in runs.iter().enumerate() {
            assert_eq!(
                [
                    run.calls.plans,
                    run.calls.keys,
                    run.calls.gathers,
                    run.calls.kernels
                ],
                case.calls,
                "{}, decision {index}",
                case.name
            );
            assert_eq!(
                run.outcome, fresh[index].outcome,
                "{}: bits, charge and stop",
                case.name
            );
            let prior = if case.stopped_prior {
                None
            } else if index == 0 {
                Some(&case.before)
            } else {
                Some(&case.after)
            };
            assert_eq!(
                run.tree_runs
                    .iter()
                    .map(|r| r.metrics().recombined_nodes)
                    .sum::<u128>(),
                tree::expected_nodes(&now, prior),
                "{}: tree law",
                case.name
            );
            if case.stopped_prior {
                assert_eq!(run.runs, [Run::Full(Cause::Stopped)]);
            }
            if let Some(write) = case.own_write {
                now.written(write);
            }
        }
    }
}
