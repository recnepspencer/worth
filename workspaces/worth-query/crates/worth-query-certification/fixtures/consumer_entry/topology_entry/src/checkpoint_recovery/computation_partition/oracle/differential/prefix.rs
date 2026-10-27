//! Lifecycle steps begin after a seeded prefix; complete interleaving belongs to 6.12.
use super::*;

pub(in super::super) fn model() -> Model {
    Model::new(&mut Lcg(SEED ^ 0x69))
}

pub(super) fn apply<const WORK: usize, const RUNS: usize, const MODE: u8>(
    application: &Application<false, WORK, RUNS, MODE>,
    changes: Vec<Change>,
    command: &mut u64,
) {
    let (scope, principal) = authenticate(application);
    let request = application.request(&principal, &scope);
    for change in changes {
        *command += 1;
        match change {
            Change::Entry(change) => edit(&request, application, change, *command),
            Change::Ordinate(y) => adjust(&request, application, y, *command),
        }
    }
}

pub(in super::super) fn run<const MODE: u8>(
    application: &Application<false, TOTALS_WORK, 1, MODE>,
) -> (Model, Vec<OracleRun>) {
    let (model, runs, _) = run_with_history(application);
    (model, runs)
}

pub(in super::super) fn run_with_history<const MODE: u8>(
    application: &Application<false, TOTALS_WORK, 1, MODE>,
) -> (Model, Vec<OracleRun>, reference::Reference) {
    let mut model = model();
    let mut history = reference::Reference::new(&model);
    let mut rng = Lcg(SEED ^ 0x6969);
    let mut kinds = KINDS;
    for place in (1..kinds.len()).rev() {
        kinds.swap(place, rng.below(place + 1));
    }
    let (scope, principal) = authenticate(application);
    let request = application.request(&principal, &scope);
    let mut last = demand(&request, application).1;
    history.demanded(None);
    let mut command = 0x6900;
    for kind in kinds.into_iter().take(8).flat_map(|kind| match kind {
        Kind::Fault => vec![Kind::Fault, Kind::Repair],
        kind => vec![kind],
    }) {
        let step = model.step(kind, &mut rng);
        for change in &step.changes {
            history.edit(change);
        }
        apply(application, step.changes, &mut command);
        arm_own_write(step.own_write);
        let next = demand(&request, application).1;
        if !next.is_empty() {
            last = next;
        }
        arm_own_write(None);
        history.demanded(step.own_write);
        if let Some(write) = step.own_write {
            model.written(write);
        }
    }
    assert!(
        last.last().unwrap().outcome.is_ok(),
        "the seeded prefix ends after repair"
    );
    (model, last, history)
}

pub(in super::super) fn value_edit(model: &Model) -> EntryEdit {
    at_demand_scope(EntryEdit::new(
        "even",
        model.first_number(),
        super::super::super::entry_edit::EntryFact::Value,
        2.5_f64.to_bits(),
    ))
}
