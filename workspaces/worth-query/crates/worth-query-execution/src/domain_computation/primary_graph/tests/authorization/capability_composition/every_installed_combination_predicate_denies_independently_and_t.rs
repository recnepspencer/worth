//! every installed combination predicate denies independently and together.
use super::*;

#[test]
fn every_installed_combination_predicate_denies_independently_and_together() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        for (scenario, expected_causes, expected_explanation) in [
            (
                CapabilityCompositionScenario::MissingAssignment,
                vec![WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing],
                WorthQueryApplicationAuthorizationExplanationCause::MissingCapability,
            ),
            (
                CapabilityCompositionScenario::ExplicitDeny,
                vec![WorthQueryOperationAuthorizationDenialKind::ExplicitDenyRuleMatched],
                WorthQueryApplicationAuthorizationExplanationCause::ExplicitPolicyDenial,
            ),
            (
                CapabilityCompositionScenario::ConflictingBeneficiary,
                vec![WorthQueryOperationAuthorizationDenialKind::ConflictRuleMatched],
                WorthQueryApplicationAuthorizationExplanationCause::Conflict,
            ),
            (
                CapabilityCompositionScenario::RequestActor,
                vec![WorthQueryOperationAuthorizationDenialKind::SeparationOfDutyRuleMatched],
                WorthQueryApplicationAuthorizationExplanationCause::SeparationOfDuty,
            ),
            (
                CapabilityCompositionScenario::PriorActor,
                vec![WorthQueryOperationAuthorizationDenialKind::DistinctActorRuleMatched],
                WorthQueryApplicationAuthorizationExplanationCause::SeparationOfDuty,
            ),
            (
                CapabilityCompositionScenario::AccumulatedProhibitions,
                vec![
                    WorthQueryOperationAuthorizationDenialKind::ExplicitDenyRuleMatched,
                    WorthQueryOperationAuthorizationDenialKind::ConflictRuleMatched,
                    WorthQueryOperationAuthorizationDenialKind::SeparationOfDutyRuleMatched,
                    WorthQueryOperationAuthorizationDenialKind::DistinctActorRuleMatched,
                ],
                WorthQueryApplicationAuthorizationExplanationCause::ExplicitPolicyDenial,
            ),
        ] {
            let world = installed_composed_capability_world(scenario);
            world.authorization_time.script([time(100)]);
            let request = live_scope();
            let principal = authenticated_principal(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &request,
            );

            let denial = admit_composed_access(
                &host_custody.phase_for_application(&world.application),
                &world,
                &principal,
                &request,
            )
            .err()
            .unwrap_or_else(|| panic!("{scenario:?} must deny at initial access admission"));

            assert!(denial.identity().is_some(), "{scenario:?}");
            assert_eq!(denial.kind(), expected_causes[0], "{scenario:?}");
            assert_eq!(denial.causes(), expected_causes, "{scenario:?}");
            assert_eq!(
                denial.explanation_cause(),
                Some(expected_explanation),
                "{scenario:?}"
            );
        }
    })
}
