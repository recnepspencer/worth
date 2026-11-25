//! Changed canonical source data can compute the same capped output.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
fn prepare(host: &world::DirHost<program::Lazy>, expected: snapshot::Snapshot) {
    let cap_scope = request_scope();
    let cap_principal = authenticate_operator(host.installed_schema(), &cap_scope);
    let request = host.request(&cap_principal, &cap_scope);
    let cap = request
        .advanced()
        .mutate(member_edit::Intent {
            key: "m".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"cap-generated-count-at-two".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(cap, WorthQueryApplicationMutationOutcome::Committed { .. }),
        "{cap:?}"
    );
    verify_adoption(host, expected);
    let source_scope = request_scope();
    let source_principal = authenticate_operator(host.installed_schema(), &source_scope);
    let request = host.request(&source_principal, &source_scope);
    let changed = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"request-three-under-cap-two".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{changed:?}"
    );
    assert_eq!(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        3
    );
}

fn verify_adoption(host: &world::DirHost<program::Lazy>, expected: snapshot::Snapshot) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    adoption::verify(host, &request, expected);
}

fn run(worker: bool, cold: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let (host, expected) = courts::build_legacy_rows(worker);
    prepare(&host, expected.clone());
    if cold {
        let directory = host.checkpoint_and_close_for_reopen();
        let host = world::reopen_program_directory::<program::Lazy>(directory, worker, false);
        verify_adoption(&host, expected);
    } else {
        verify_adoption(&host, expected);
    }
}
#[test]
fn a_changed_canonical_source_adopts_equal_output() {
    run(false, false);
}
#[test]
fn worker_only_changed_canonical_source_adopts_equal_output() {
    run(true, false);
}
#[test]
fn cold_changed_canonical_source_adopts_equal_output() {
    run(false, true);
}
#[test]
fn worker_only_cold_changed_canonical_source_adopts_equal_output() {
    run(true, true);
}
