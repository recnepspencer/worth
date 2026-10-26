//! Whole-orbit disclosure is indivisible and uses a genuinely issued capability.
use super::*;
#[test]
fn governed_orbit_allows_its_exact_slot_and_refuses_omission_or_missing_rule() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = world();
    let allowed = read::run(&world, Allowed::reference(), 0);
    assert_eq!(
        allowed
            .orbit(whole::<OrbitSchema<true>, Allowed>())
            .unwrap()
            .len(),
        3
    );
    let host = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let error = read::execute(&world, &execution, &request, Omitted::reference(), 0)
        .err()
        .expect("the query must refuse");
    assert!(
        matches!(error, read::Stop::Read(ref error) if error.kind() == WorthQueryApplicationOneShotDenialKind::ProjectionUnavailable && error.subject() == "orbit"),
        "{error:?}"
    );
    let error = read::execute(&world, &execution, &request, Missing::reference(), 0)
        .err()
        .expect("the query must refuse");
    assert!(
        matches!(error, read::Stop::Admission(ref error) if error.kind() == WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{error:?}"
    );
}
