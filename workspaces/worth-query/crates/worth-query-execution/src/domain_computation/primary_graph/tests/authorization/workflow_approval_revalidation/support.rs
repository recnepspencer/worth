use super::super::super::fixture::capability::{CapabilityGrant, CapabilityStatusField};
use super::super::capability_delegation_mutation::grant;
use super::*;
use worth_relational::facade::authorization::{
    RelationalAuthorizationObservationPlan, RelationalAuthorizationPathPlan,
    RelationalAuthorizationPredicate,
};

#[test]
fn copied_primary_dependencies_cannot_forge_delegation_target_support() {
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
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

#[test]
fn delegation_support_comparator_rejects_independent_native_source_aba() {
    let world =
        super::super::super::fixture::installed_delegated_capability_world_with_unrelated(1);
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let authority = admission.workflow_approval_authority().unwrap();
    // This comparator court uses a separate real native source. It does not
    // claim production delegation issuance.
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
    let mut dependencies: serde_json::Value = serde_json::from_str(
        &admission
            .workflow_approval_dependencies_snapshot()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    dependencies["support"] = dependencies["primary"].clone();
    dependencies["support"]["activation"] = native_stamp_for_independent_grant(&world).into();
    let dependency_snapshot = dependencies.to_string();
    let basis = durable_approval_basis_with_dependency_snapshot(
        &admission,
        Some(&support),
        None,
        Some(&dependency_snapshot),
    );
    world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .expect("an issued native source stamp and fresh active target must agree");

    for mutated in [
        support.replace("\"active\"", "\"upper_bound\""),
        support.replace("delegation_target", "elevation_upper_bound"),
        support.replace(
            &authority.capability_authority_identity,
            "uninstalled-support-authority",
        ),
    ] {
        let altered = durable_approval_basis_with_dependency_snapshot(
            &admission,
            Some(&mutated),
            None,
            Some(&dependency_snapshot),
        );
        let denial = world
            .application
            .readmit_workflow_approval_authority(&altered, &admission)
            .unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
        );
    }
    let mut wrong_resource: serde_json::Value = serde_json::from_str(&support).unwrap();
    let mut support_request: serde_json::Value =
        serde_json::from_str(wrong_resource["request"].as_str().unwrap()).unwrap();
    support_request["resource"] = serde_json::to_value(authority.grant).unwrap();
    wrong_resource["request"] = serde_json::Value::String(support_request.to_string());
    let altered = durable_approval_basis_with_dependency_snapshot(
        &admission,
        Some(&wrong_resource.to_string()),
        None,
        Some(&dependency_snapshot),
    );
    let denial = world
        .application
        .readmit_workflow_approval_authority(&altered, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::RelationalObservationRejected
    );

    update_grant_field(
        &world,
        "unrelated-capability-0",
        field(&world, CapabilityStatusField::reference()),
        super::super::super::fixture::CapabilityStatusBinding::encode(
            &super::super::super::fixture::CapabilityStatus::Revoked,
        )
        .unwrap(),
    );
    update_grant_field(
        &world,
        "unrelated-capability-0",
        field(&world, CapabilityStatusField::reference()),
        super::super::super::fixture::CapabilityStatusBinding::encode(
            &super::super::super::fixture::CapabilityStatus::Active,
        )
        .unwrap(),
    );
    let fresh_principal = authenticated_principal(&world, &request);
    let fresh = admitted_capability_operation(&world, &fresh_principal, &request);
    let original_dependencies: serde_json::Value = serde_json::from_str(
        &admission
            .workflow_approval_dependencies_snapshot()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let fresh_dependencies: serde_json::Value = serde_json::from_str(
        &fresh
            .workflow_approval_dependencies_snapshot()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        original_dependencies["primary"],
        fresh_dependencies["primary"]
    );
    assert_eq!(
        admission.workflow_approval_lineage_snapshot().unwrap(),
        fresh.workflow_approval_lineage_snapshot().unwrap()
    );
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &fresh)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

fn native_stamp_for_independent_grant(
    world: &super::super::super::fixture::AuthorizationWorld,
) -> String {
    let selected = world.selected_product();
    let snapshot = selected.application_basis().snapshot_handle().clone();
    let graph = world.application.runtime.primary_graph().unwrap();
    let grant = grant(world, "unrelated-capability-0");
    let kind = graph
        .layout()
        .entity_kind(CapabilityGrant::reference().name())
        .unwrap();
    let predicate = RelationalAuthorizationPredicate::new(
        0,
        kind,
        field(world, CapabilityStatusField::reference()),
        super::super::super::fixture::CapabilityStatusBinding::encode(
            &super::super::super::fixture::CapabilityStatus::Active,
        )
        .unwrap(),
    );
    graph.integration_handle().with_runtime_mut(|runtime| {
        let plan = RelationalAuthorizationObservationPlan::try_new(
            snapshot,
            grant,
            grant,
            kind,
            kind,
            [RelationalAuthorizationPathPlan::new([], [predicate])],
            [],
        )
        .unwrap();
        let observation = runtime.observe_authorization(plan).unwrap();
        assert!(observation.paths()[0].matched());
        runtime
            .capture_authorization_durable_dependencies(&observation)
            .unwrap()
            .to_wire_string()
            .unwrap()
    })
}

#[test]
fn expired_retained_support_is_denied_before_native_replay() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let authority = admission.workflow_approval_authority().unwrap();
    assert!(matches!(
        authority.expiry,
        worth_foundational::facade::AspectValue::UInt64(primary_expiry) if primary_expiry > 108
    ));
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
    let mut dependencies: serde_json::Value = serde_json::from_str(
        &admission
            .workflow_approval_dependencies_snapshot()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    dependencies["support"] = dependencies["primary"].clone();
    dependencies["support"]["activation"] = dependencies["primary"]["primary"].clone();
    let basis = durable_approval_basis_with_dependency_snapshot(
        &admission,
        Some(&support),
        None,
        Some(&dependencies.to_string()),
    );
    world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .expect("support is admitted before its retained expiry");
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
