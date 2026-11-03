use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::output_source_observation::SourceQueryObservationStop;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::output_source_observation::SourceQueryPreparationPhase;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;

#[test]
fn authorized_cardinality_outcomes_preserve_handler_choice() {
    assert!(!SourceQueryObservationStop::Missing.is_control_refusal());
    assert!(!SourceQueryObservationStop::Ambiguous.is_control_refusal());
}

#[test]
fn control_contention_cannot_be_swallowed_or_fail_a_shared_output() {
    let stop = WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::CurrentControlBusy,
        "current control",
    );
    assert_eq!(stop.recovery_posture(),
            worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture::Retryable);
    assert!(SourceQueryObservationStop::CurrentControl(stop).is_control_refusal());
}

#[test]
fn permission_and_budget_refusals_are_not_authored_data_outcomes() {
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind as Kind;
    for kind in [
        Kind::StalePrincipal,
        Kind::DisclosureGovernanceRequired,
        Kind::WorkLimitExceeded,
    ] {
        assert!(SourceQueryObservationStop::Preparation {
            phase: SourceQueryPreparationPhase::Permission,
            cause: WorthQueryApplicationQueryAdmissionDenial::new(kind, "source query"),
        }
        .is_control_refusal());
    }
}
