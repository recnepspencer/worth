use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::direct_attempt;

#[test]
fn direct_attempt_mints_one_session_and_binds_immutable_evidence() {
    let attempt = direct_attempt("direct", 8);

    assert_eq!(attempt.resources().counters().provider_session_mints, 1);
    assert_eq!(
        attempt.provider_session_for_test().attempt_identity(),
        attempt.attempt_identity().as_str()
    );
    assert_eq!(
        attempt.evidence().admission_identity(),
        attempt.resources().identity()
    );
    assert_eq!(
        attempt.evidence().provider_session_identity(),
        attempt.provider_session_for_test().identity()
    );
    assert_eq!(
        attempt.evidence().provider_session_attempt_identity(),
        attempt.provider_session_for_test().attempt_identity()
    );
}
#[test]
fn repeated_direct_attempts_receive_unique_provider_sessions() {
    let first = direct_attempt("repeat", 8);
    let second = direct_attempt("repeat", 8);

    assert_ne!(
        first.provider_session_for_test().attempt_identity(),
        second.provider_session_for_test().attempt_identity()
    );
    assert_eq!(
        first.provider_session_for_test().attempt_identity(),
        first.attempt_identity().as_str()
    );
    assert_eq!(
        second.provider_session_for_test().attempt_identity(),
        second.attempt_identity().as_str()
    );
    assert_ne!(
        first.provider_session_for_test().identity(),
        second.provider_session_for_test().identity()
    );
    assert_ne!(first.evidence().identity(), second.evidence().identity());
}
