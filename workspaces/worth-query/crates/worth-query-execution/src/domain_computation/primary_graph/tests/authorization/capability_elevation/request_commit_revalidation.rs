use super::super::super::application_attempt::{authenticated_principal, idempotency};
use super::super::super::fixture::{
    installed_elevated_capability_world, live_scope, revoke_current_capability,
    CapabilityElevationIdentity, CapabilityElevationScenario, CapabilityReviewIdentity,
};
use super::super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryElevationRequestOutcome, WorthQueryEntityResolutionDenialKind,
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn governed_upper_bound_support_is_revalidated_at_request_commit() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 8));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let program = super::request_transition::request_reads(
        &world,
        &principal,
        &request,
        super::request_transition::honest_input(),
    )
    .materialize_elevation_request_program()
    .unwrap();

    revoke_current_capability(&world);

    let WorthQueryElevationRequestOutcome::Denied(denial) = world
        .application
        .compare_and_commit_elevation_request(program, idempotency(74, 74))
    else {
        panic!("revoked upper-bound support must deny before lifecycle effects");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::DecisionReadSet
    );
}

#[test]
fn provider_time_rejects_a_request_program_after_its_exact_window_expires() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 8));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let program = super::request_transition::request_reads(
        &world,
        &principal,
        &request,
        super::request_transition::honest_input(),
    )
    .materialize_elevation_request_program()
    .unwrap();

    world
        .authorization_time
        .script(std::iter::repeat_n(time(106), 4));
    let WorthQueryElevationRequestOutcome::Denied(denial) = world
        .application
        .compare_and_commit_elevation_request(program, idempotency(75, 75))
    else {
        panic!("provider time must deny an expired request program without minting a receipt");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::DecisionReadSet
    );

    let elevation = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            "elevation-2".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap_err();
    assert_eq!(
        elevation.kind(),
        WorthQueryEntityResolutionDenialKind::UnknownEntity
    );
    let review = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityReviewIdentity::reference(),
            "review-2".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap_err();
    assert_eq!(
        review.kind(),
        WorthQueryEntityResolutionDenialKind::UnknownEntity
    );
}
