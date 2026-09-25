use super::*;

#[test]
fn durable_approval_readmission_checks_retained_support() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let authority = admission.workflow_approval_authority().unwrap();
    let support = serde_json::json!({
        "request": admission.workflow_approval_request_snapshot().unwrap().unwrap(),
        "grant": authority.grant,
        "capability_authority_identity": authority.capability_authority_identity,
        "posture": "active",
        "role": "delegation_target",
        "timeline": authority.timeline.canonical_name(),
        "expiry": 300,
        "lineage": serde_json::from_str::<serde_json::Value>(
            &admission.workflow_approval_lineage_snapshot().unwrap().unwrap()
        ).unwrap(),
    })
    .to_string();
    let basis = durable_approval_basis_with_snapshots(&admission, Some(&support), None);
    world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .expect("retained active support must be readmitted");
    let upper_bound = support.replace("\"active\"", "\"upper_bound\"");
    let upper_bound = durable_approval_basis_with_snapshots(&admission, Some(&upper_bound), None);
    let denial = world
        .application
        .readmit_workflow_approval_authority(&upper_bound, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    let wrong_role = support.replace("delegation_target", "elevation_upper_bound");
    let wrong_role = durable_approval_basis_with_snapshots(&admission, Some(&wrong_role), None);
    let denial = world
        .application
        .readmit_workflow_approval_authority(&wrong_role, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    let mut wrong_resource: serde_json::Value = serde_json::from_str(&support).unwrap();
    let mut support_request: serde_json::Value =
        serde_json::from_str(wrong_resource["request"].as_str().unwrap()).unwrap();
    support_request["resource"] = serde_json::to_value(authority.grant).unwrap();
    wrong_resource["request"] = serde_json::Value::String(support_request.to_string());
    let wrong_resource =
        durable_approval_basis_with_snapshots(&admission, Some(&wrong_resource.to_string()), None);
    let denial = world
        .application
        .readmit_workflow_approval_authority(&wrong_resource, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    let altered = support.replace(
        &authority.capability_authority_identity,
        "uninstalled-support-authority",
    );
    let altered = durable_approval_basis_with_snapshots(&admission, Some(&altered), None);
    let denial = world
        .application
        .readmit_workflow_approval_authority(&altered, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

#[test]
fn retained_support_expiry_cannot_be_revived_by_a_current_grant() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let authority = admission.workflow_approval_authority().unwrap();
    let support = serde_json::json!({
        "request": admission.workflow_approval_request_snapshot().unwrap().unwrap(),
        "grant": authority.grant,
        "capability_authority_identity": authority.capability_authority_identity,
        "posture": "active",
        "role": "delegation_target",
        "lineage": serde_json::from_str::<serde_json::Value>(
            &admission.workflow_approval_lineage_snapshot().unwrap().unwrap()
        ).unwrap(),
        "timeline": authority.timeline.canonical_name(),
        "expiry": 105,
    })
    .to_string();
    let basis = durable_approval_basis_with_snapshots(&admission, Some(&support), None);
    world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .expect("support remains valid before its retained expiry");
    world.authorization_time.hold(time(108));
    let principal = authenticated_principal(&world, &request);
    let fresh = admitted_capability_operation(&world, &principal, &request);
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &fresh)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityExpired
    );
}
