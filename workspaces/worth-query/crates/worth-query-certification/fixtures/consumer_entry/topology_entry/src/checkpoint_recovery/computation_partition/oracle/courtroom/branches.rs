//! The edit alphabet on independently evolving forks, with model call laws.
use super::super::differential::alphabet::{Change, Lcg, KINDS};
use super::*;
use worth_query_host::facade::product::WorthQueryProductBranch as Branch;

fn fork<const MODE: u8>(app: &Application<false, TOTALS_WORK, 1, MODE>, parent: Branch) -> Branch {
    app.branches()
        .fork(parent)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .unwrap()
}
fn run<const MODE: u8>(
    app: &Application<false, TOTALS_WORK, 1, MODE>,
    branch: Branch,
) -> (usize, Vec<OracleRun>) {
    let (scope, principal) = authenticate(app);
    demand(&app.request(&principal, &scope).on_branch(branch), app)
}
fn apply<const MODE: u8>(
    app: &Application<false, TOTALS_WORK, 1, MODE>,
    branch: Branch,
    changes: &[Change],
    command: &mut u64,
) {
    let (scope, principal) = authenticate(app);
    let request = app.request(&principal, &scope).on_branch(branch);
    for change in changes {
        *command += 1;
        match change {
            Change::Entry(change) => edit(&request, app, change.clone(), *command),
            Change::Ordinate(y) => adjust(&request, app, *y, *command),
        }
    }
}

fn judge(
    model: &Model,
    prior: &mut Option<Model>,
    kept: &[OracleRun],
    fresh: &[OracleRun],
    write: Option<super::super::super::region_output::OwnWrite>,
) {
    assert_eq!(
        kept.iter().map(|run| &run.outcome).collect::<Vec<_>>(),
        fresh.iter().map(|run| &run.outcome).collect::<Vec<_>>()
    );
    if !kept.is_empty() {
        assert_published_state(kept, fresh);
    }
    let mut now = model.clone();
    for (index, run) in kept.iter().enumerate() {
        assert_eq!(
            run.calls,
            now.expected_calls(prior.as_ref()),
            "branch decision {index}"
        );
        assert_eq!(
            run.tree_runs
                .iter()
                .map(|tree| tree.metrics().recombined_nodes)
                .sum::<u128>(),
            tree::expected_nodes(&now, prior.as_ref())
        );
        *prior = now.completes().then(|| now.clone());
        if index == 0 {
            if let Some(write) = write {
                now.written(write);
            }
        }
    }
}

#[test]
fn seeded_branch_alphabet_has_exact_counts_on_forks_switches_and_deletion() {
    let _guard = checkpoint_recovery_test_guard();
    for seed in SEEDS {
        let mut rng = Lcg(seed);
        let model = Model::new(&mut rng);
        let kept = installation::install_variant::<false, TOTALS_WORK, 1, 0>(
            None,
            Default::default(),
            |graph| model.seed(graph),
        );
        let fresh = installation::install_variant::<false, TOTALS_WORK, 1, 3>(
            None,
            Default::default(),
            |graph| model.seed(graph),
        );
        let main = kept.current_world();
        let full_main = fresh.current_world();
        let mut prior = None;
        let (contacts, a) = run(&kept, main);
        let (_, b) = run(&fresh, full_main);
        assert_eq!(contacts, 1);
        judge(&model, &mut prior, &a, &b, None);
        let parent = fork(&kept, main);
        let full_parent = fork(&fresh, full_main);
        let child = fork(&kept, parent);
        let full_child = fork(&fresh, full_parent);
        let nested = fork(&kept, child);
        let full_nested = fork(&fresh, full_child);
        let branches = [parent, child, nested];
        let full_branches = [full_parent, full_child, full_nested];
        let mut models = [model.clone(), model.clone(), model];
        let mut priors = [prior.clone(), prior.clone(), prior];
        for index in 0..branches.len() {
            let (contacts, a) = run(&kept, branches[index]);
            let (_, b) = run(&fresh, full_branches[index]);
            // A fresh fork has no demand row; its producer runs once, carrying all partitions.
            assert_eq!(contacts, 1);
            judge(&models[index], &mut priors[index], &a, &b, None);
            assert_eq!(a[0].calls, OwnerCalls::default());
        }
        let mut command = 0x612_0000;
        for _ in 0..ROUNDS {
            let mut order = KINDS;
            for place in (1..order.len()).rev() {
                order.swap(place, rng.below(place + 1));
            }
            for kind in order {
                let index = rng.below(branches.len());
                for kind in match kind {
                    Kind::Fault => vec![kind, Kind::Repair],
                    Kind::Ceiling => vec![kind, Kind::Relief],
                    kind => vec![kind],
                } {
                    let step = models[index].step(kind, &mut rng);
                    apply(&kept, branches[index], &step.changes, &mut command);
                    apply(&fresh, full_branches[index], &step.changes, &mut command);
                    super::super::super::region_output::arm_own_write(step.own_write);
                    let (contacts, a) = run(&kept, branches[index]);
                    let (_, b) = run(&fresh, full_branches[index]);
                    super::super::super::region_output::arm_own_write(None);
                    let executes = kind != Kind::NoOp;
                    let written = executes
                        && models[index].completes()
                        && step
                            .own_write
                            .is_some_and(|write| !models[index].holds(write));
                    assert_eq!(
                        (contacts, a.len()),
                        (
                            usize::from(executes) + usize::from(written),
                            usize::from(executes) + usize::from(written)
                        )
                    );
                    judge(&models[index], &mut priors[index], &a, &b, step.own_write);
                    if let Some(write) = step.own_write {
                        models[index].written(write);
                    }
                    let sibling = (index + 1) % branches.len();
                    let (contacts, a) = run(&kept, branches[sibling]);
                    let (_, b) = run(&fresh, full_branches[sibling]);
                    assert_eq!((contacts, a.len(), b.len()), (0, 0, 0), "unchanged switch");
                }
            }
        }
        // Closing ancestors cannot release the nested branch's retained basis.
        for (a, b) in [(child, full_child), (parent, full_parent)] {
            let _kept_cleanup = super::super::branch_sharing::close_ancestor(&kept, a);
            let _fresh_cleanup = super::super::branch_sharing::close_ancestor(&fresh, b);
            let step = models[2].step(Kind::Value, &mut rng);
            apply(&kept, nested, &step.changes, &mut command);
            apply(&fresh, full_nested, &step.changes, &mut command);
            let (contacts, a) = run(&kept, nested);
            let (_, b) = run(&fresh, full_nested);
            assert_eq!(contacts, 1);
            judge(&models[2], &mut priors[2], &a, &b, None);
        }
    }
}
