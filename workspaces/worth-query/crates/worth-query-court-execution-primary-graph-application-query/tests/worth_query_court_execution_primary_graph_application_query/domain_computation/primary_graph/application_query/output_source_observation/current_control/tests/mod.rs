use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::output_source_observation::SourceQueryObservationStop;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::output_source_observation::SourceQueryPreparationPhase;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenial;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;

#[test]
fn authorized_cardinality_outcomes_preserve_handler_choice() {
    assert!(!SourceQueryObservationStop::Missing.is_control_refusal_for_test());
    assert!(!SourceQueryObservationStop::Ambiguous.is_control_refusal_for_test());
}

#[test]
fn control_contention_cannot_be_swallowed_or_fail_a_shared_output() {
    let stop = WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::CurrentControlBusy,
        "current control",
    );
    assert_eq!(stop.recovery_posture(),
            worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture::Retryable);
    assert!(SourceQueryObservationStop::CurrentControl(stop).is_control_refusal_for_test());
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
            cause: WorthQueryApplicationQueryAdmissionDenial::new_for_test(kind, "source query"),
        }
        .is_control_refusal_for_test());
    }
}

#[test]
fn historical_source_presence_refusals_cannot_be_swallowed() {
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind as Kind;
    use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
    let source = EntityId::new(PartitionId::main(), 7, 2);
    for kind in [
        Kind::HistoricalSourceNotLive { source },
        Kind::HistoricalSourceKindMismatch {
            source,
            actual: KindId::new(3),
        },
        Kind::HistoricalSourceOutsideDomain { source },
    ] {
        let cause = WorthQueryApplicationQueryAdmissionDenial::new_for_test(kind, "source query");
        assert!(SourceQueryObservationStop::Admission(cause.clone()).is_control_refusal_for_test());
        assert!(SourceQueryObservationStop::Preparation {
            phase: SourceQueryPreparationPhase::Permission,
            cause,
        }
        .is_control_refusal_for_test());
    }
}
