//! Durable approval authority is descriptive until the owner readmits it.

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_live_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_delegated_capability_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_commit_revalidation::disable_mapping;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_commit_revalidation::enable_mapping;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_commit_revalidation::revoke_grant;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::field;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::retarget_relation_without_replacing_id;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::update_grant_field;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_progression::revoke_parent;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_operation;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::Admission;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
pub use worth_query_installation::facade::ApplicationScalarValueBinding;

pub use crate::domain_computation::primary_graph::tests::authorization::workflow_approval_revalidation::durable_basis::durable_approval_basis;
pub use crate::domain_computation::primary_graph::tests::authorization::workflow_approval_revalidation::durable_basis::durable_approval_basis_with_dependency_snapshot;
pub use crate::domain_computation::primary_graph::tests::authorization::workflow_approval_revalidation::durable_basis::durable_approval_basis_with_snapshots;

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

#[path = "approver_reenable_does_not_revive_older_approval/mod.rs"]
pub mod approver_reenable_does_not_revive_older_approval;
#[path = "durable_approval_readmission_denies_disabled_approver/mod.rs"]
pub mod durable_approval_readmission_denies_disabled_approver;
#[path = "durable_basis/mod.rs"]
pub mod durable_basis;
#[path = "field_text/mod.rs"]
pub mod field_text;
#[path = "grant_reactivation_does_not_revive_older_approval/mod.rs"]
pub mod grant_reactivation_does_not_revive_older_approval;
#[path = "later_grant_extension_cannot_extend_the_original_approval/mod.rs"]
pub mod later_grant_extension_cannot_extend_the_original_approval;
#[path = "native/mod.rs"]
pub mod native;
#[path = "parent_reactivation_does_not_revive_delegated_approval/mod.rs"]
pub mod parent_reactivation_does_not_revive_delegated_approval;
#[path = "principal_target_retarget_aba_does_not_revive_older_approval/mod.rs"]
pub mod principal_target_retarget_aba_does_not_revive_older_approval;
#[path = "read_stop_oracle/mod.rs"]
pub mod read_stop_oracle;
#[path = "support/mod.rs"]
pub mod support;
