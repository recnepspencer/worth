#[test]
fn stale_provider_session_cause_survives_plan_and_commit_projection() {
    use crate::domain_computation::{
        WorthQueryProviderSessionDenialKind as Kind, WorthQueryProviderSessionFailure as Failure,
        WorthQueryProviderSessionProtocolCounters as Counters,
        WorthQueryProviderSessionProtocolStage as Protocol,
    };
    for (project, stage) in [
        (
            super::provider_session_admission_denied as fn(Failure) -> super::Denial,
            super::Stage::ProviderPlan,
        ),
        (
            super::provider_session_denied as fn(Failure) -> super::Denial,
            super::Stage::ProviderCommit,
        ),
    ] {
        let denial = project(Failure::new(
            Kind::StaleIncarnation,
            Protocol::PlanReadmission,
            "stale session",
            Counters::default(),
        ));
        assert_eq!(
            denial.execution_denial_cause(),
            Some(Ok(Kind::StaleIncarnation))
        );
        assert_eq!(denial.stage(), stage);
    }
}
