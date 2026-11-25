//! A cold stale input uses genuine Native custody, executes once and adopts.
use super::*;
fn run(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let (host, expected) = courts::build_legacy_rows(worker);
    {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        adoption::raise_cap(&host, &request);
    }
    let directory = host.checkpoint_and_close_for_reopen();
    let host = world::reopen_program_directory::<program::Lazy>(directory, worker, false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    assert_eq!(
        request
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0],
        expected
    );
    adoption::verify(&host, &request, expected);
}
#[test]
fn cold_changed_input_executes_once_without_publication_then_replays_current() {
    run(false);
}
#[test]
fn worker_only_cold_changed_input_executes_once_without_publication_then_replays_current() {
    run(true);
}
