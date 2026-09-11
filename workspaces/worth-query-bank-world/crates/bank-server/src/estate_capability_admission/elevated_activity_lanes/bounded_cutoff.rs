use super::support::{
    activity_request, activity_world, assert_exact_revoked_alternate_active,
    assert_resources_released, controls, revoke_exact_support,
};
use crate::BankApplicationQueryDenial;

#[test]
fn historical_activity_denies_exact_support_loss_after_admission() {
    let world = activity_world("estate-emergency-activity-historical-cutoff");
    let result = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(controls(8))
        .admit_historical_with_approved_elevation(&world.approved, |admitted| {
            revoke_exact_support(&world, 157);
            admitted.execute()
        });
    let Err(denial) = result else {
        panic!("historical activity must refresh exact support before payload delivery");
    };

    let BankApplicationQueryDenial::Execution(denial) = denial else {
        panic!("historical cutoff must occur during selected execution: {denial:?}");
    };
    assert_stale_authorization(denial.kind());
    assert_exact_revoked_alternate_active(&world);
    assert_resources_released(&world);
}

#[test]
fn preview_activity_denies_exact_support_loss_after_admission() {
    let world = activity_world("estate-emergency-activity-preview-cutoff");
    let session = world
        .fixture
        .runtime
        .open_preview(&super::super::fixture::request_scope())
        .unwrap();
    let result = world
        .fixture
        .runtime
        .query(activity_request())
        .as_principal(&world.requester)
        .controls(controls(8))
        .admit_preview_with_approved_elevation(&world.approved, &session, |admitted| {
            revoke_exact_support(&world, 159);
            admitted.execute()
        });
    let Err(denial) = result else {
        panic!("preview activity must refresh exact support before payload delivery");
    };

    let BankApplicationQueryDenial::Execution(denial) = denial else {
        panic!("preview cutoff must occur during selected execution: {denial:?}");
    };
    assert_stale_authorization(denial.kind());
    assert_exact_revoked_alternate_active(&world);
    assert!(session.discard().unwrap().discarded());
    assert_resources_released(&world);
}

fn assert_stale_authorization(kind: crate::BankApplicationOneShotDenialKind) {
    assert_eq!(
        kind,
        crate::BankApplicationOneShotDenialKind::Authorization(
            crate::BankAuthorizationDenialKind::StaleAuthorization,
        )
    );
}
