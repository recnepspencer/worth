//! One-call demands and admitted edits of the neutral application.
use super::*;

/// Seeds the entry numbered `number` as a member of each of `sets`.
pub(super) fn seed_entry(graph: &mut Graph, sets: &[&str], number: usize, entry: RegionEntry) {
    let key = format!("oracle-entry-{number}");
    facts::seed_entry(graph, &key, &entry);
    for set in sets {
        facts::seed_member(graph, set, &key);
    }
}

/// Demands the scope's output in one advance: the producer's contacts in
/// the demand, and the runs its decisions made.
pub(super) fn demand<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    request: &Request<'_, '_, '_>,
    application: &Application<REUSE, WORK, RUNS, MODE>,
) -> (usize, Vec<OracleRun>) {
    demand_reconciled(request, application, None)
}

pub(super) fn demand_reconciled<
    const REUSE: bool,
    const WORK: usize,
    const RUNS: usize,
    const MODE: u8,
>(
    request: &Request<'_, '_, '_>,
    application: &Application<REUSE, WORK, RUNS, MODE>,
    serial_tree: Option<
        &[worth_query_host::facade::application_contribution::WorthQueryPartitionedTreeRun],
    >,
) -> (usize, Vec<OracleRun>) {
    room().clear();
    published_states();
    let mut demand = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram<REUSE, WORK, RUNS, MODE>, RegionConnection>(
            application,
        )
        .expect("the region output demand starts");
    let mut advances = 0;
    let mut advance = || {
        advances += 1;
        demand.advance(request).map_err(Box::new)
    };
    let progress = advance();
    assert_eq!(advances, 1, "a demand is judged after exactly one advance");
    let settled = match progress.unwrap_or_else(|denial| {
        panic!(
            "the region output demand advances: {denial:?}; runs: {:?}",
            *room()
        )
    }) {
        WorthQueryApplicationOutputDemandProgress::Settled(settled) => settled,
        WorthQueryApplicationOutputDemandProgress::Pending => {
            panic!("one advance must settle; decisions: {:?}", *room())
        }
    };
    let contacts = settled.producer_contacts_in_this_demand();
    let mut runs = take_runs(serial_tree);
    let published = published_states();
    if let Some(last) = runs.last_mut() {
        last.published = published;
    } else {
        assert!(
            published.is_empty(),
            "a demand with no run publishes nothing"
        );
    }
    (contacts, runs)
}

/// Commits one entry edit.
pub(super) fn edit<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    request: &Request<'_, '_, '_>,
    application: &Application<REUSE, WORK, RUNS, MODE>,
    edit: EntryEdit,
    command: u64,
) {
    let made = format!("{edit:?}");
    let outcome = request
        .mutate(edit.commanded(command))
        .without_source()
        .idempotency(&command)
        .execute_in_program::<OracleProgram<REUSE, WORK, RUNS, MODE>>(application);
    assert!(
        matches!(
            &outcome,
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "the edit {made} commits: {outcome:?}"
    );
}

/// Moves the scope's ordinate to `y`, which names the set its output totals.
pub(super) fn adjust<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    request: &Request<'_, '_, '_>,
    application: &Application<REUSE, WORK, RUNS, MODE>,
    y: u64,
    command: u64,
) {
    let observed = request
        .query(PlanarRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .expect("the scope's source is readable");
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: SCOPE.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&command)
        .execute_performed::<OracleProgram<REUSE, WORK, RUNS, MODE>, OracleRoot>(application)
        .expect("the scope's ordinate moves");
}

pub(super) fn at_demand_scope(mut edit: EntryEdit) -> EntryEdit {
    edit.scope_key = SCOPE.to_owned();
    edit
}
