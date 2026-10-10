use super::super::super::application_attempt::idempotency;
use super::super::super::fixture::{CapabilityElevationStatus, CapabilityReviewStatus};
use super::super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryElevationCloseOutcome, WorthQueryElevationClosureKind,
};

#[test]
fn provider_time_rejects_stale_revoked_classification_then_commits_exact_expired_close() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, approved) =
            super::approval_transition::exact_approved_world(&host_custody);
        world
            .authorization_time
            .script(std::iter::repeat_n(time(100), 16));
        let program = super::terminal_lifecycle_support::materialize_close(
            &host_custody,
            &world,
            &request,
            approved,
        );

        world
            .authorization_time
            .script(std::iter::repeat_n(time(106), 16));
        let WorthQueryElevationCloseOutcome::Denied(denial, approved) = world
            .application
            .compare_and_commit_elevation_close(program, idempotency(177, 177))
        else {
            panic!("provider time must reject a Revoked close after the exact window expires");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ProviderRejected
        );
        assert_eq!(
            denial.stage(),
            WorthQueryApplicationCommitDenialStage::DecisionReadSet
        );
        assert_eq!(
            super::terminal_state::elevation_status(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &host_custody.borrowed_request_for_test(),
                &world
            ),
            CapabilityElevationStatus::Approved
        );
        assert_eq!(
            super::terminal_state::review_status(
                &(host_custody).test_host_request(),
                &host_custody.borrowed_request_for_test(),
                &world
            ),
            CapabilityReviewStatus::Required
        );

        world
            .authorization_time
            .script(std::iter::repeat_n(time(106), 16));
        let program = super::terminal_lifecycle_support::materialize_close(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
            &world,
            &request,
            approved,
        );
        let WorthQueryElevationCloseOutcome::Closed(mandatory) = world
            .application
            .compare_and_commit_elevation_close(program, idempotency(178, 178))
        else {
            panic!("a freshly classified Expired close must commit");
        };
        assert_eq!(
            mandatory.closure_kind(),
            WorthQueryElevationClosureKind::Expired
        );
        assert_eq!(mandatory.closed_at(), &time_value(106));
        assert_eq!(
            super::terminal_state::elevation_status(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &host_custody.borrowed_request_for_test(),
                &world
            ),
            CapabilityElevationStatus::Expired
        );
    })
}

fn time_value(seconds: u64) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::UInt64(seconds)
}
