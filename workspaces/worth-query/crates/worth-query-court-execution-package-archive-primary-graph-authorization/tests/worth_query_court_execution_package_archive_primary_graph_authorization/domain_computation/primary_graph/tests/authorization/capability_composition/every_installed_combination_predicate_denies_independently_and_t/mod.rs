//! every installed combination predicate denies independently and together.
use crate::domain_computation::primary_graph::tests::authorization::capability_composition::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn every_installed_combination_predicate_denies_independently_and_together() {
    for (scenario, expected_causes, expected_explanation) in [
        (
            CapabilityCompositionScenario::MissingAssignment,
            vec![WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::CapabilityAuthorizationMissing,
            )],
            WorthQueryApplicationAuthorizationExplanationCause::MissingCapability,
        ),
        (
            CapabilityCompositionScenario::ExplicitDeny,
            vec![WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ExplicitDenyRuleMatched,
            )],
            WorthQueryApplicationAuthorizationExplanationCause::ExplicitPolicyDenial,
        ),
        (
            CapabilityCompositionScenario::ConflictingBeneficiary,
            vec![WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ConflictRuleMatched,
            )],
            WorthQueryApplicationAuthorizationExplanationCause::Conflict,
        ),
        (
            CapabilityCompositionScenario::RequestActor,
            vec![WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::SeparationOfDutyRuleMatched,
            )],
            WorthQueryApplicationAuthorizationExplanationCause::SeparationOfDuty,
        ),
        (
            CapabilityCompositionScenario::PriorActor,
            vec![WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::DistinctActorRuleMatched,
            )],
            WorthQueryApplicationAuthorizationExplanationCause::SeparationOfDuty,
        ),
        (
            CapabilityCompositionScenario::AccumulatedProhibitions,
            vec![
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::ExplicitDenyRuleMatched,
                ),
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::ConflictRuleMatched,
                ),
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::SeparationOfDutyRuleMatched,
                ),
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::DistinctActorRuleMatched,
                ),
            ],
            WorthQueryApplicationAuthorizationExplanationCause::ExplicitPolicyDenial,
        ),
    ] {
        let world = installed_composed_capability_world(scenario);
        world.authorization_time.script([time(100)]);
        let request = live_scope();
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                authenticated_principal(
                    &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");

        let denial = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                admit_composed_access(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    &principal,
                    &request,
                )
                .err()
                .unwrap_or_else(|| panic!("{scenario:?} must deny at initial access admission"))
            })
            .expect("the fixture read owns its installed host call");

        assert!(denial.identity().is_some(), "{scenario:?}");
        assert_eq!(denial.kind(), expected_causes[0], "{scenario:?}");
        assert_eq!(denial.causes(), expected_causes, "{scenario:?}");
        assert_eq!(
            denial.explanation_cause(),
            Some(expected_explanation),
            "{scenario:?}"
        );
    }
}
