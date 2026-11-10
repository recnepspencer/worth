//! What still refuses after the shrink fix (public #126).
//!
//! A shrink whose request stops inside the rebase after its commit leaves the
//! writer's record factless and without a witness. Hot adoption answers such a
//! record from its own commit, which cannot decide, and refuses
//! `RetainedBasisUnavailable` before any producer contact, as it did before the
//! fix. A demand whose own request stops refuses `Cancelled`, also before any
//! contact.
use super::*;
use worth_execution::with_request_consult_observer_for_test;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

pub(super) type Host = WorthQueryProgramApplicationRuntime<Schema, program::Synchronous>;

/// Consults counted back from the end of the shrink's unstopped run. A stop
/// there lands inside the rebase after the commit: the measured window runs
/// from about 516 to about 220 back hot (`d1506-undecidable-window.log`) and
/// about 468 to 216 back on the worker path (`d1506-kept-refusals.log`, WSCAN
/// lines), and the count moves by about 20 between runs.
const INSIDE_THE_REBASE: usize = 370;

pub(super) fn settled_host(worker: bool) -> Host {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<program::Synchronous>(limits, 5);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    assert_eq!(demand(&host, &request).0, Ok(()));
    host
}

/// Run `body` under a request that stops at consult `stop_at`; also return
/// the consults it made.
pub(super) fn stoppable<R>(
    host: &Host,
    stop_at: Option<usize>,
    body: impl FnOnce(&family::Request<'_, '_, '_>) -> R,
) -> (R, usize) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        WorthQueryAdmissionClock::system(),
    );
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    with_request_consult_observer_for_test(
        move |at| {
            if Some(at) == stop_at {
                cancellation.cancel();
            }
        },
        || body(&request),
    )
}

/// The edit to `value`: whether it performed, its consults, and its commits.
pub(super) fn edit(host: &Host, value: u64, stop_at: Option<usize>) -> (bool, usize, usize) {
    let ((performed, consults), work) = observe_maintenance_work(64, || {
        stoppable(host, stop_at, |request| {
            matches!(
                request
                    .advanced()
                    .mutate(edit::Intent {
                        key: "a".into(),
                        value,
                    })
                    .without_source()
                    .idempotency(&"kept-refusals".to_owned())
                    .execute_performed::<program::Synchronous, program::Tree>(
                        host,
                        ExecutionAllocationPolicy::SystemAllocation,
                    ),
                Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(_))
            )
        })
    });
    (
        performed,
        consults,
        work.published_rows.len() + work.omitted_publications,
    )
}

/// A fresh demand settled or refused (rendered), with its producer contacts
/// and commits.
pub(super) fn demand(
    host: &Host,
    request: &family::Request<'_, '_, '_>,
) -> (Result<(), String>, u64, usize) {
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (outcome, work) = observe_maintenance_work(64, || {
        let mut outputs = request
            .advanced()
            .start_program_outputs::<program::Synchronous, program::Tree>(
                host,
                Demand("a".into()),
                WorthQueryOutputDemandControls::default(),
            )
            .map_err(|cause| format!("{cause:?}"))?;
        for _ in 0..128 {
            match outputs.advance(host, request) {
                Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => return Ok(()),
                Ok(_) => {}
                Err(cause) => return Err(format!("{cause:?}")),
            }
        }
        panic!("the demand settles or refuses")
    });
    (
        outcome,
        host.producer_contacts_on_this_thread_for_test() - contacts,
        work.published_rows.len() + work.omitted_publications,
    )
}

/// Each completed record's idempotency identity and its (keeps comparable
/// facts, has a sealed witness).
fn adoption_bases(host: &Host) -> Vec<([u8; 32], (bool, bool))> {
    let graph = host
        .runtime()
        .runtime_for_test()
        .primary_provider_for_test()
        .graph_for_test();
    let lineage = graph.output_lineage.lock().unwrap();
    let mut bases = Vec::new();
    for branches in lineage.by_source_for_test().values() {
        for generations in branches.values() {
            for records in generations.values() {
                bases.extend(records.iter().filter_map(|record| {
                    let record = record.get()?;
                    Some((
                        *record.idempotency_key_identity_for_test(),
                        record.adoption_basis_for_test()?,
                    ))
                }));
            }
        }
    }
    bases
}

/// The bases of the records `edit` completes: those under an idempotency
/// identity the host did not hold before.
fn edit_bases(host: &Host, edit: impl FnOnce()) -> Vec<(bool, bool)> {
    let before: std::collections::BTreeSet<_> = adoption_bases(host)
        .into_iter()
        .map(|(identity, _)| identity)
        .collect();
    edit();
    let all = adoption_bases(host);
    all.into_iter()
        .filter(|(identity, _)| !before.contains(identity))
        .map(|(_, basis)| basis)
        .collect()
}

/// On both paths the stop lands after the World commit, so the edit reports
/// `Performed` (public #132: the worker path reported a refusal before any
/// effect).
fn undecided_rebase(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let (performed, total, commits) = edit(&settled_host(worker), 2, None);
    assert!(
        performed && commits == 1,
        "the unstopped shrink is one commit"
    );
    let host = settled_host(worker);
    let mut stopped = (false, 0, 0);
    let bases = edit_bases(&host, || {
        stopped = edit(&host, 2, Some(total - INSIDE_THE_REBASE))
    });
    let (performed, consults, commits) = stopped;
    assert!(
        consults >= total - INSIDE_THE_REBASE,
        "the stop fired: {consults} consults"
    );
    assert!(
        performed,
        "the stop lands after the World commit, which it cannot undo"
    );
    assert_eq!(commits, 1);
    assert!(
        bases.contains(&(false, false)),
        "the edit leaves a record with no comparable facts and no witness: {bases:?}"
    );
    let ((refused, contacts, commits), _) =
        stoppable(&host, None, |request| demand(&host, request));
    let refused = refused.expect_err("the undecidable record is not adopted");
    // The typed kind is nested in the handle's error; its rendering is stable.
    assert!(
        refused.contains("kind: RetainedBasisUnavailable"),
        "{refused}"
    );
    assert_eq!(
        (contacts, commits),
        (0, 0),
        "refused before any producer contact"
    );
}

fn stopped_demand(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let grown = |worker| {
        let host = settled_host(worker);
        let mut grown = (false, 0, 0);
        let bases = edit_bases(&host, || grown = edit(&host, 7, None));
        let (performed, _, commits) = grown;
        assert!(performed && commits == 1, "the growing edit is one commit");
        assert!(
            bases.contains(&(true, true)),
            "the edit leaves a record with facts and a witness: {bases:?}"
        );
        host
    };
    let host = grown(worker);
    let ((adopted, contacts, commits), consults) =
        stoppable(&host, None, |request| demand(&host, request));
    assert_eq!((adopted, contacts, commits), (Ok(()), 0, 0), "hot adoption");
    for stop_at in [1, consults / 2, consults] {
        let host = grown(worker);
        let ((refused, contacts, commits), _) =
            stoppable(&host, Some(stop_at), |request| demand(&host, request));
        let refused = refused.expect_err("a stopped demand does not settle");
        assert!(
            refused.contains("Cancelled"),
            "stop at {stop_at}: {refused}"
        );
        assert_eq!((contacts, commits), (0, 0), "stop at {stop_at}");
    }
}

#[test]
fn a_shrink_stopped_inside_its_rebase_still_refuses_its_undecidable_record() {
    undecided_rebase(false);
}
#[test]
fn worker_only_shrink_stopped_inside_its_rebase_still_refuses_its_undecidable_record() {
    undecided_rebase(true);
}
#[test]
fn a_demand_stopped_by_its_request_still_refuses_before_any_contact() {
    stopped_demand(false);
}
#[test]
fn worker_only_demand_stopped_by_its_request_still_refuses_before_any_contact() {
    stopped_demand(true);
}
