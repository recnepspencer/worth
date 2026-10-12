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
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
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
    durable_approval_basis_with_dependency_snapshot(
        admission,
        replacement_support,
        replacement_lineage,
        None,
    )
}

fn durable_approval_basis_with_dependency_snapshot(
    admission: &Admission,
    replacement_support: Option<&str>,
    replacement_lineage: Option<&str>,
    replacement_dependencies: Option<&str>,
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
    let original_dependencies = admission
        .workflow_approval_dependencies_snapshot()
        .unwrap()
        .unwrap();
    let mut dependency_snapshot: serde_json::Value =
        serde_json::from_str(replacement_dependencies.unwrap_or(&original_dependencies)).unwrap();
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
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let original = admission
        .workflow_approval_lineage_snapshot()
        .unwrap()
        .unwrap();
    assert!(original.contains("Root"));
    let changed = "\"Unbound\"";
    let basis = durable_approval_basis_with_snapshots(&admission, None, Some(changed));
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .readmit_workflow_approval_authority(
                    &host_custody.borrowed_request_for_test(),
                    &basis,
                    &admission,
                )
                .unwrap_err()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization
        )
    );
}

#[test]
fn durable_approval_readmission_denies_revoked_grant() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let basis = durable_approval_basis(&admission);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .readmit_workflow_approval_authority(
                    &host_custody.borrowed_request_for_test(),
                    &basis,
                    &admission,
                )
                .expect("unchanged durable approval authority must readmit")
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| revoke_grant(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .readmit_workflow_approval_authority(
                    &host_custody.borrowed_request_for_test(),
                    &basis,
                    &admission,
                )
                .unwrap_err()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing
        )
    );
}

#[path = "workflow_approval_revalidation/grant_reactivation_does_not_revive_older_approval.rs"]
mod grant_reactivation_does_not_revive_older_approval;

#[path = "workflow_approval_revalidation/durable_approval_readmission_denies_disabled_approver.rs"]
mod durable_approval_readmission_denies_disabled_approver;

#[path = "workflow_approval_revalidation/approver_reenable_does_not_revive_older_approval.rs"]
mod approver_reenable_does_not_revive_older_approval;

#[path = "workflow_approval_revalidation/principal_target_retarget_aba_does_not_revive_older_approval.rs"]
mod principal_target_retarget_aba_does_not_revive_older_approval;

#[test]
fn durable_approval_readmission_denies_expired_grant() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let basis = durable_approval_basis(&admission);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .readmit_workflow_approval_authority(
                    &host_custody.borrowed_request_for_test(),
                    &basis,
                    &admission,
                )
                .expect("unchanged durable approval authority must readmit")
        })
        .expect("the fixture read owns its installed host call");
    world.authorization_time.hold(time(300));
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .readmit_workflow_approval_authority(
                    &host_custody.borrowed_request_for_test(),
                    &basis,
                    &admission,
                )
                .unwrap_err()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityExpired
        )
    );
}

#[path = "workflow_approval_revalidation/later_grant_extension_cannot_extend_the_original_approval.rs"]
mod later_grant_extension_cannot_extend_the_original_approval;

#[test]
fn durable_approval_readmission_denies_revoked_delegation_parent() {
    let world = installed_delegated_capability_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let basis = durable_approval_basis(&admission);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .readmit_workflow_approval_authority(
                    &host_custody.borrowed_request_for_test(),
                    &basis,
                    &admission,
                )
                .expect("unchanged delegated approval authority must readmit")
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| revoke_parent(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .readmit_workflow_approval_authority(
                    &host_custody.borrowed_request_for_test(),
                    &basis,
                    &admission,
                )
                .unwrap_err()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::DelegationRejected
        )
    );
}

#[path = "workflow_approval_revalidation/parent_reactivation_does_not_revive_delegated_approval.rs"]
mod parent_reactivation_does_not_revive_delegated_approval;
