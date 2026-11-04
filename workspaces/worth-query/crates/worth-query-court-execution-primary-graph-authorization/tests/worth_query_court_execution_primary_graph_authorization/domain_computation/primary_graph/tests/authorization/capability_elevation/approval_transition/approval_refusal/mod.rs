//! The requester and a conflicted approver cannot enter approval progression.
use super::*;

#[test]
fn requester_and_conflicted_approver_cannot_enter_approval_progression() {
    for (scenario, subject, expected) in [
        (
            CapabilityElevationScenario::Active,
            "alice",
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::CapabilityAuthorizationMissing,
            ),
        ),
        (
            CapabilityElevationScenario::ConflictedApprover,
            "bob",
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ConflictRuleMatched,
            ),
        ),
    ] {
        let (world, request, _requested) = requested_world(scenario);
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                authenticated(&host_custody, &world, subject, &request)
            })
            .expect("the fixture read owns its installed host call");
        let denial = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                approval_access(&host_custody, &world, &principal, &request)
                    .err()
                    .expect("actor policy must deny before lifecycle authority")
            })
            .expect("the fixture read owns its installed host call");
        assert_eq!(denial.kind(), expected);
        assert!(denial.identity().is_some());
        assert_eq!(denial.causes(), [expected]);
    }
}
