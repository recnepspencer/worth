//! Native authorization revisions prevent value-equivalent readmission after ABA.

use super::*;

#[test]
fn grant_expiry_aba_does_not_revive_older_approval() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    let expiry_field = field(
        &world,
        super::super::super::fixture::capability::CapabilityNotAfterField::reference(),
    );
    update_grant_field(
        &world,
        "capability-1",
        expiry_field.clone(),
        worth_foundational::facade::AspectValue::UInt64(105),
    );
    update_grant_field(
        &world,
        "capability-1",
        expiry_field,
        worth_foundational::facade::AspectValue::UInt64(110),
    );
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}
