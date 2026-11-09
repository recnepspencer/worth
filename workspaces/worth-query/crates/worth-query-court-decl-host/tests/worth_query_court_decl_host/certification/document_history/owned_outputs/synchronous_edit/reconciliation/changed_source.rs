//! Changed canonical source data can compute the same capped output.
use super::super::failure;
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
fn prepare(
    host: &WorthQueryProgramApplicationRuntime<Schema, program::Lazy>,
    request: &family::Request<'_, '_, '_>,
    expected: snapshot::Snapshot,
) {
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
    adoption::verify(host, request, expected);
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
fn run(worker: bool, cold: bool) {
    let mut saved = None;
    courts::with_legacy_rows(worker, |host, request, expected| {
        prepare(host, request, expected.clone());
        if cold {
            saved = Some((failure::checkpoint(host), expected));
        } else {
            adoption::verify(host, request, expected);
        }
    });
    if let Some((checkpoint, expected)) = saved {
        let _observation = WorthQueryExecutionObservation::install_on_this_thread();
        let host = world::restore_program_checkpoint::<program::Lazy>(checkpoint, worker, false);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        adoption::verify(&host, &request, expected);
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
