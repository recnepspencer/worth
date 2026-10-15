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
    decode_workflow_approval_support, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRetainedCapabilityRequest, WorthQueryWorkflowApprovalAuthorityBasis,
};
use worth_query_installation::facade::ApplicationScalarValueBinding;

#[path = "workflow_approval_revalidation/native.rs"]
mod native;
#[path = "workflow_approval_revalidation/support.rs"]
mod support;

mod durable_basis;
use durable_basis::{
    durable_approval_basis, durable_approval_basis_with_dependency_snapshot,
    durable_approval_basis_with_snapshots,
};

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
    let original = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            admission
                .workflow_approval_lineage_snapshot(&host.borrowed_request_for_test())
                .unwrap()
                .unwrap()
        })
        .expect("the fixture envelope owns its installed host call");
    assert!(original.contains("Root"));
    let changed = "\"Unbound\"";
    let basis = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            durable_approval_basis_with_snapshots(
                &host.borrowed_request_for_test(),
                &admission,
                None,
                Some(changed),
            )
        })
        .expect("the fixture envelope owns its installed host call");
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
    let basis = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            durable_approval_basis(&host.borrowed_request_for_test(), &admission)
        })
        .expect("the fixture envelope owns its installed host call");
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
    let basis = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            durable_approval_basis(&host.borrowed_request_for_test(), &admission)
        })
        .expect("the fixture envelope owns its installed host call");
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
    let basis = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            durable_approval_basis(&host.borrowed_request_for_test(), &admission)
        })
        .expect("the fixture envelope owns its installed host call");
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

#[path = "workflow_approval_revalidation/read_stop_oracle.rs"]
mod read_stop_oracle;
