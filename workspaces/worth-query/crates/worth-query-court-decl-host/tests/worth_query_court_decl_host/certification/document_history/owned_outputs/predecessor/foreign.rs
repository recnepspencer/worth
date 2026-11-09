//! Historical foreign values neither become current dependencies nor disappear at export.
use super::courts::{reset, settle};
use super::*;
fn mutate(host: &Host, request: &family::Request<'_, '_, '_>, key: &str, value: u64) {
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: key.into(),
            value,
        })
        .without_source()
        .idempotency(&format!("predecessor-foreign-{key}-{value}"))
        .execute_in_program::<program::Program>(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
}
#[test]
fn an_old_foreign_value_is_not_a_current_dependency_and_current_output_reopens() {
    let checkpoint = {
        let host = world::install_program_with_other_model::<program::Program>(host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, Some(10));
        reset(false);
        PROBE.with(|probe| probe.borrow_mut().foreign = true);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        settle(&request, &host).unwrap();
        mutate(&host, &request, "b", 20);
        mutate(&host, &request, "a", 2);
        settle(&request, &host).unwrap();
        assert_eq!(
            request
                .query(query::model::Read("b".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            20
        );
        PROBE.with(|probe| assert_eq!(probe.borrow().values.last(), Some(&Ok(Some(10)))));
        let reads = PROBE.with(|probe| probe.borrow().values.len());
        mutate(&host, &request, "b", 30);
        settle(&request, &host).unwrap();
        PROBE.with(|probe| {
            assert_eq!(
                probe.borrow().values.len(),
                reads,
                "historical B.Value must never become a current output dependency"
            )
        });
        host.with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap()
    };
    let host = world::restore_program_checkpoint::<program::Program>(checkpoint, false, false);
    reset(false);
    PROBE.with(|probe| probe.borrow_mut().foreign = true);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&request, &host).unwrap();
    // SourceReadAuthorization is deliberately nonreusable in checkpoints. Its
    // owner reruns this producer; cold predecessor roots remain unavailable.
    PROBE.with(|probe| {
        assert_eq!(
            probe.borrow().values.as_slice(),
            &[Err(
                WorthQueryPredecessorReadDenial::PredecessorBasisUnavailable
            )],
            "cold fallback must never substitute a current B.Value for old truth"
        )
    });
    assert_eq!(
        request
            .query(query::first_output::Read("predecessor:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        9002
    );
}
