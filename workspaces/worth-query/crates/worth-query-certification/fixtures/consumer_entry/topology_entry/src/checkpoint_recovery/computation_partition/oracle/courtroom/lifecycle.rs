//! Checkpoint restoration and generated republication inside the edit alphabet.
use super::super::differential::alphabet::{Change, Lcg, KINDS};
use super::super::differential::reference::Reference;
use super::*;

fn apply(
    app: &Application<false, TOTALS_WORK, 1, 2>,
    changes: &[Change],
    history: &mut Reference,
    command: &mut u64,
) {
    let (scope, principal) = authenticate(app);
    let request = app.request(&principal, &scope);
    for change in changes {
        history.edit(change);
        *command += 1;
        match change {
            Change::Entry(change) => edit(&request, app, change.clone(), *command),
            Change::Ordinate(y) => adjust(&request, app, *y, *command),
        }
    }
}

fn judge(
    app: &Application<false, TOTALS_WORK, 1, 2>,
    model: &Model,
    history: &mut Reference,
    prior: &mut Option<Model>,
    last_facts: &mut Option<Model>,
    write: Option<super::super::super::region_output::OwnWrite>,
    cause: Option<Cause>,
) -> Vec<OracleRun> {
    let fresh = history.install::<false, TOTALS_WORK, 1, 2>(Default::default(), |g, m| m.seed(g));
    let (scope, principal) = authenticate(app);
    let (fs, fp) = authenticate(&fresh);
    super::super::super::region_output::arm_own_write(write);
    let (contacts, kept) = demand(&app.request(&principal, &scope), app);
    super::super::super::region_output::arm_own_write(write);
    let (_, reference) = demand(&fresh.request(&fp, &fs), &fresh);
    super::super::super::region_output::arm_own_write(None);
    let executes = last_facts.as_ref() != Some(model) || cause.is_some();
    let writes = executes && model.completes() && write.is_some_and(|w| !model.holds(w));
    let decisions = usize::from(executes) + usize::from(writes);
    assert_eq!((contacts, kept.len()), (decisions, decisions));
    if !kept.is_empty() {
        assert_eq!(
            kept.iter().map(|r| &r.outcome).collect::<Vec<_>>(),
            reference.iter().map(|r| &r.outcome).collect::<Vec<_>>()
        );
        assert_published_state(&kept, &reference);
    }
    let mut now = model.clone();
    for (index, run) in kept.iter().enumerate() {
        assert_eq!(run.calls, now.expected_calls(prior.as_ref()));
        assert_eq!(
            run.tree_runs
                .iter()
                .map(|t| t.metrics().recombined_nodes)
                .sum::<u128>(),
            tree::expected_nodes(&now, prior.as_ref())
        );
        if let Some(cause) = cause {
            assert_eq!(run.runs, [Run::Full(cause)]);
        }
        *prior = now.completes().then(|| now.clone());
        if index == 0 {
            if let Some(write) = write {
                now.written(write);
            }
        }
    }
    *last_facts = Some(now);
    history.demanded(write);
    kept
}

#[test]
fn seeded_edits_cross_restore_and_republication_with_model_counts() {
    let _guard = checkpoint_recovery_test_guard();
    for seed in SEEDS {
        let mut rng = Lcg(seed);
        let mut model = Model::new(&mut rng);
        let mut history = Reference::new(&model);
        let mut app = installation::install_variant::<false, TOTALS_WORK, 1, 2>(
            None,
            Default::default(),
            |g| model.seed(g),
        );
        let mut prior = None;
        let mut last_facts = None;
        judge(
            &app,
            &model,
            &mut history,
            &mut prior,
            &mut last_facts,
            None,
            Some(Cause::FirstRun),
        );
        let mut command = 0x612_11fe;
        for _ in 0..ROUNDS {
            let mut kinds = KINDS;
            for place in (1..kinds.len()).rev() {
                kinds.swap(place, rng.below(place + 1));
            }
            for kind in kinds.into_iter().flat_map(|kind| match kind {
                Kind::Fault => vec![kind, Kind::Repair],
                Kind::Ceiling => vec![kind, Kind::Relief],
                kind => vec![kind],
            }) {
                let step = model.step(kind, &mut rng);
                apply(&app, &step.changes, &mut history, &mut command);
                judge(
                    &app,
                    &model,
                    &mut history,
                    &mut prior,
                    &mut last_facts,
                    step.own_write,
                    None,
                );
                if let Some(write) = step.own_write {
                    model.written(write);
                }
            }
            let checkpoint = app.capture_application_checkpoint(worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy::SystemAllocation).unwrap();
            drop(app);
            app = installation::install_variant::<false, TOTALS_WORK, 1, 2>(
                Some(checkpoint),
                Default::default(),
                |_| panic!("restore does not reseed"),
            );
            prior = None;
            let step = model.step(Kind::Value, &mut rng);
            apply(&app, &step.changes, &mut history, &mut command);
            let restored = judge(
                &app,
                &model,
                &mut history,
                &mut prior,
                &mut last_facts,
                None,
                Some(Cause::Restored),
            );
            assert_eq!(restored.len(), 1);
            let total = restored[0].outcome.as_ref().unwrap().0;
            super::super::republication::republish(&app, total);
            prior = None;
            let step = model.step(Kind::Value, &mut rng);
            apply(&app, &step.changes, &mut history, &mut command);
            let republished = judge(
                &app,
                &model,
                &mut history,
                &mut prior,
                &mut last_facts,
                None,
                Some(Cause::Republished),
            );
            assert_eq!(republished.len(), 1);
        }
    }
}
