//! Whole-slot omission refuses before Native acquisition; guards retain separate authority.
use super::*;
#[test]
fn governed_components_refuse_closed_slots_before_native_and_guard_reads() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = interleaved_world();
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let count=crate::domain_computation::primary_graph::application_query::component_native_acquisitions_on_this_thread_for_test;
    let before = count();
    let allowed = read::run(&world, Allowed::reference(), 0);
    assert_eq!(
        allowed
            .components(selection::<OrbitSchema<true>, Allowed>())
            .unwrap()
            .len(),
        2
    );
    assert!(
        count() > before,
        "positive control must enter original Native acquisition"
    );
    let guarded = read::run(&world, GuardAllowed::reference(), 0);
    assert!(guarded.rows().is_empty());
    assert!(
        guarded
            .components(selection::<OrbitSchema<true>, GuardAllowed>())
            .unwrap()
            .is_empty(),
        "an admitted false guard yields genuine complete empty evidence"
    );
    let host = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let before = count();
    let denied = read::execute(&world, &execution, &request, Omitted::reference(), 0)
        .err()
        .unwrap();
    assert!(
        matches!(denied,read::Stop::Read(ref stop) if stop.kind()==WorthQueryApplicationOneShotDenialKind::ProjectionUnavailable && stop.subject()=="shells"),
        "{denied:?}"
    );
    assert_eq!(count(), before);
    let denied = read::execute(&world, &execution, &request, Missing::reference(), 0)
        .err()
        .unwrap();
    assert!(
        matches!(denied,read::Stop::Admission(ref stop) if stop.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{denied:?}"
    );
    assert_eq!(count(), before);
    let denied = read::execute(&world, &execution, &request, GuardDenied::reference(), 0)
        .err()
        .unwrap();
    assert!(
        matches!(denied,read::Stop::Admission(ref stop) if stop.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{denied:?}"
    );
    assert_eq!(
        count(),
        before,
        "guard authorization precedes census cohort acquisition"
    );
    let denied = read::execute(
        &world,
        &execution,
        &request,
        AggregateDenied::reference(),
        0,
    )
    .err()
    .unwrap();
    assert!(
        matches!(denied,read::Stop::Admission(ref stop) if stop.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{denied:?}"
    );
    assert_eq!(
        count(),
        before,
        "aggregate identity permission is independently required"
    );
    let denied = read::execute(&world, &execution, &request, OrderingDenied::reference(), 0)
        .err()
        .unwrap();
    assert!(
        matches!(denied,read::Stop::Admission(ref stop) if stop.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{denied:?}"
    );
    assert_eq!(
        count(),
        before,
        "component ordering permission is independently required"
    );
}
