//! A cold stale input uses genuine Native custody, executes once and adopts.
use super::super::failure;
use super::*;
fn run(worker: bool) {
    let mut retained = None;
    courts::with_legacy_rows(worker, |host, request, expected| {
        adoption::raise_cap(host, request);
        retained = Some((failure::checkpoint(host), expected));
    });
    let (checkpoint, expected) = retained.unwrap();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::restore_program_checkpoint::<program::Lazy>(checkpoint, worker, false);
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
