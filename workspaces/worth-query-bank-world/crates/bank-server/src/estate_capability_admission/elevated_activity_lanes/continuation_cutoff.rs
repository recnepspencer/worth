use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryResumeControls;

use super::support::{
    activity_request, activity_world, assert_exact_revoked_alternate_active,
    assert_resources_released, controls, revoke_exact_support,
};
use crate::BankApplicationQueryDenial;

#[test]
fn exact_support_loss_after_continuation_readmission_denies_before_payload() {
    let world = activity_world("estate-emergency-activity-continuation-cutoff");
    let first = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(controls(1))
        .page_with_approved_elevation(&world.approved)
        .expect("two real emergency accesses should mint a continuation");
    assert_eq!(super::support::items(first.rows()).len(), 1);
    let (_, continuation) = first.into_parts();
    let continuation = continuation.expect("the many emergency relation must have a next page");
    let request = super::super::fixture::request_scope();
    let outcome = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(controls(1))
        .readmit_resume_with_approved_elevation(
            &world.approved,
            continuation,
            WorthQueryApplicationQueryResumeControls::new(
                std::num::NonZeroUsize::new(1).unwrap(),
                std::num::NonZeroUsize::new(20_000).unwrap(),
                &request,
            ),
            |admitted| {
                revoke_exact_support(&world, 151);
                admitted.execute()
            },
        );
    let denial = match outcome {
        Ok(_) => panic!("post-readmission exact-support loss returned a payload"),
        Err(denial) => denial,
    };
    let BankApplicationQueryDenial::ContinuationExecution(denial) = denial else {
        panic!("the cutoff must occur inside bounded continuation execution: {denial:?}");
    };
    assert_eq!(
        denial.kind(),
        crate::BankApplicationContinuationDenialKind::Authorization(
            crate::BankAuthorizationDenialKind::StaleAuthorization,
        )
    );
    assert_exact_revoked_alternate_active(&world);
    assert_resources_released(&world);
}

#[test]
fn foreign_approval_cannot_open_or_resume_activity_pages() {
    let world = activity_world("estate-emergency-activity-page-foreign-approval");
    let foreign = activity_world("estate-emergency-activity-page-other-approval");
    let denied_page = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(controls(1))
        .page_with_approved_elevation(&foreign.approved);
    let Err(BankApplicationQueryDenial::CapabilityAdmission(denial)) = denied_page else {
        panic!("foreign approval must not open an activity page");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::ElevationApprovalRejected
    );

    let first = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(controls(1))
        .page_with_approved_elevation(&world.approved)
        .expect("own approval opens the first page");
    let (_, continuation) = first.into_parts();
    let continuation = continuation.expect("the second access should require another page");
    let request = super::super::fixture::request_scope();
    let denied_resume = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(controls(1))
        .resume_with_approved_elevation(
            &foreign.approved,
            continuation,
            WorthQueryApplicationQueryResumeControls::new(
                std::num::NonZeroUsize::new(1).unwrap(),
                std::num::NonZeroUsize::new(20_000).unwrap(),
                &request,
            ),
        );
    let Err(BankApplicationQueryDenial::CapabilityAdmission(denial)) = denied_resume else {
        panic!("foreign approval must not resume an activity page");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::ElevationApprovalRejected
    );
}
