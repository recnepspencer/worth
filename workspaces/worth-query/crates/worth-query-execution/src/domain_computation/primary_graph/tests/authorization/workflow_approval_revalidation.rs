//! Durable approval authority is descriptive until the owner readmits it.

use super::super::application_attempt::authenticated_principal;
use super::super::fixture::{
    installed_capability_authorization_world, installed_capability_live_world,
    installed_delegated_capability_world, live_scope,
};
use super::capability_commit_revalidation::{disable_mapping, enable_mapping, revoke_grant};
use super::capability_delegation_mutation::{
    field, retarget_relation_without_replacing_id, update_grant_field,
};
use super::capability_delegation_progression::revoke_parent;
use super::capability_progression::{admitted_capability_operation, time, Admission};
use crate::domain_computation::authorization::{
    decode_workflow_approval_support, WorthQueryDurableCapabilityLineage,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryRetainedCapabilityRequest,
    WorthQueryWorkflowApprovalAuthorityBasis,
};
use crate::domain_computation::primary_graph::WorthQueryDurablePrincipalCurrentness;
use worth_query_installation::facade::ApplicationScalarValueBinding;

fn durable_approval_basis(admission: &Admission) -> WorthQueryWorkflowApprovalAuthorityBasis {
    durable_approval_basis_with_snapshots(admission, None, None)
}

fn durable_approval_basis_with_snapshots(
    admission: &Admission,
    replacement_support: Option<&str>,
    replacement_lineage: Option<&str>,
) -> WorthQueryWorkflowApprovalAuthorityBasis {
    let authority = admission.workflow_approval_authority().unwrap();
    let request = WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
        &admission
            .workflow_approval_request_snapshot()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let principal: WorthQueryDurablePrincipalCurrentness = serde_json::from_str(
        &admission
            .workflow_approval_principal_snapshot()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let original_lineage = admission
        .workflow_approval_lineage_snapshot()
        .unwrap()
        .unwrap();
    let lineage: WorthQueryDurableCapabilityLineage =
        serde_json::from_str(replacement_lineage.unwrap_or(&original_lineage)).unwrap();
    let original_support = admission
        .workflow_approval_support_snapshot()
        .unwrap()
        .unwrap();
    let support =
        decode_workflow_approval_support(replacement_support.unwrap_or(&original_support)).unwrap();
    let mut dependency_snapshot: serde_json::Value = serde_json::from_str(
        &admission
            .workflow_approval_dependencies_snapshot()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    if support.is_some() && dependency_snapshot["support"].is_null() {
        dependency_snapshot["support"] = dependency_snapshot["primary"].clone();
    }
    let dependencies =
        crate::domain_computation::authorization::WorthQueryWorkflowApprovalDependencies::decode(
            &dependency_snapshot.to_string(),
        )
        .unwrap();
    let worth_foundational::facade::AspectValue::UInt64(expiry) = authority.expiry else {
        panic!("the fixture uses an unsigned capability expiry");
    };
    WorthQueryWorkflowApprovalAuthorityBasis::from_retained_approval(
        request,
        principal,
        authority.grant,
        authority.capability_authority_identity,
        lineage,
        support,
        dependencies,
        authority.timeline,
        expiry,
    )
}

#[path = "workflow_approval_revalidation/native.rs"]
mod native;
#[path = "workflow_approval_revalidation/support.rs"]
mod support;

#[test]
fn durable_approval_readmission_rejects_changed_lineage_shape() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let original = admission
        .workflow_approval_lineage_snapshot()
        .unwrap()
        .unwrap();
    assert!(original.contains("Root"));
    let changed = "\"Unbound\"";
    let basis = durable_approval_basis_with_snapshots(&admission, None, Some(changed));
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
fn durable_approval_readmission_denies_revoked_grant() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .expect("unchanged durable approval authority must readmit");
    revoke_grant(&world);
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing
    );
}

#[test]
fn grant_reactivation_does_not_revive_older_approval() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    revoke_grant(&world);
    update_grant_field(
        &world,
        "capability-1",
        field(
            &world,
            super::super::fixture::capability::CapabilityStatusField::reference(),
        ),
        super::super::fixture::CapabilityStatusBinding::encode(
            &super::super::fixture::CapabilityStatus::Active,
        )
        .unwrap(),
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

#[test]
fn durable_approval_readmission_denies_disabled_approver() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .expect("unchanged durable approval authority must readmit");
    disable_mapping(&world, principal.mapping_entity_id());
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StalePrincipal
    );
}

#[test]
fn approver_reenable_does_not_revive_older_approval() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    disable_mapping(&world, principal.mapping_entity_id());
    enable_mapping(&world, principal.mapping_entity_id());
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StalePrincipal
    );
}

#[test]
fn principal_target_retarget_aba_does_not_revive_older_approval() {
    let world = installed_capability_live_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    let binding = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout()
        .principal_binding(world.binding.binding())
        .unwrap();
    let other = world
        .selected_product()
        .resolve_entity(
            super::super::fixture::PrincipalIdentityField::reference(),
            2_u64,
            &request,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    retarget_relation_without_replacing_id(
        &world,
        principal.target_relation_id(),
        binding.relation_kind,
        principal.mapping_entity_id(),
        other,
    );
    retarget_relation_without_replacing_id(
        &world,
        principal.target_relation_id(),
        binding.relation_kind,
        principal.mapping_entity_id(),
        principal.principal_entity_id(),
    );
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StalePrincipal
    );
}

#[test]
fn durable_approval_readmission_denies_expired_grant() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .expect("unchanged durable approval authority must readmit");
    world.authorization_time.hold(time(300));
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityExpired
    );
}

#[test]
fn later_grant_extension_cannot_extend_the_original_approval() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    update_grant_field(
        &world,
        "capability-1",
        field(
            &world,
            super::super::fixture::capability::CapabilityNotAfterField::reference(),
        ),
        worth_foundational::facade::AspectValue::UInt64(500),
    );
    world.authorization_time.hold(time(120));
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

#[test]
fn durable_approval_readmission_denies_revoked_delegation_parent() {
    let world = installed_delegated_capability_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .expect("unchanged delegated approval authority must readmit");
    revoke_parent(&world);
    let denial = world
        .application
        .readmit_workflow_approval_authority(&basis, &admission)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::DelegationRejected
    );
}

#[test]
fn parent_reactivation_does_not_revive_delegated_approval() {
    let world = installed_delegated_capability_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let admission = admitted_capability_operation(&world, &principal, &request);
    let basis = durable_approval_basis(&admission);
    revoke_parent(&world);
    update_grant_field(
        &world,
        "capability-parent",
        field(
            &world,
            super::super::fixture::capability::CapabilityStatusField::reference(),
        ),
        super::super::fixture::CapabilityStatusBinding::encode(
            &super::super::fixture::CapabilityStatus::Active,
        )
        .unwrap(),
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
