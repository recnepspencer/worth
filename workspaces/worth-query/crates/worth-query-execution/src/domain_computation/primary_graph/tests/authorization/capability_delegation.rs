use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding, U64ApplicationValueBinding,
};

use super::super::application_attempt::authenticated_principal;
use super::super::fixture::capability::{
    CapabilityCustodian, CapabilityGrantee, CapabilityGrantor, CapabilityNotAfterField,
    CapabilityResource, CapabilityStatusField, CapabilityWorkflowField,
};
use super::super::fixture::{installed_delegated_capability_world, live_scope};
use super::capability_delegation_mutation::{
    account, add_parent, field, grant, relation_kind, relation_source, replace_parent,
    replace_relation_kind, replace_relation_source, replace_relation_target, update_grant_field,
};
use super::capability_progression::{admitted_capability_access, time};
use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind;

#[test]
fn lawful_multi_link_delegation_is_one_composite_exact_authorization_fact() {
    let world = installed_delegated_capability_world();
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    let access = admitted_capability_access(&world, &principal, &request, 100)
        .expect("the narrowed current child and exact parent must admit");

    assert_eq!(access.authorization_decision_fact_count(), 2);
    assert_eq!(access.relational_counters().paths_evaluated, 20);
    assert!(access.signal_dependency_count() > 0);
    let work = access.admission_canonical_work();
    assert_eq!(work.basis_preparations(), 0);
    assert_eq!(work.digest_derivations(), 0);
    assert_eq!(work.digest_text_materializations(), 0);
}

#[test]
fn width_and_cycle_attacks_are_typed_denials_before_access_authority() {
    let width = installed_delegated_capability_world();
    width.authorization_time.script([time(100)]);
    add_parent(
        &width,
        "capability-child",
        "capability-alternate",
        "width-parent",
    );
    let request = live_scope();
    let principal = authenticated_principal(&width, &request);
    let Err(denial) = admitted_capability_access(&width, &principal, &request, 100) else {
        panic!("two current parents must deny");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::DelegationRejected
    );

    let cycle = installed_delegated_capability_world();
    cycle.authorization_time.script([time(100)]);
    add_parent(
        &cycle,
        "capability-grandparent",
        "capability-child",
        "cycle-parent",
    );
    let request = live_scope();
    let principal = authenticated_principal(&cycle, &request);
    let Err(denial) = admitted_capability_access(&cycle, &principal, &request, 100) else {
        panic!("a delegation cycle must deny before narrowing can be reused");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::DelegationCycle
    );

    let depth = installed_delegated_capability_world();
    depth.authorization_time.script([time(100)]);
    add_parent(
        &depth,
        "capability-grandparent",
        "capability-alternate",
        "excess-depth-parent",
    );
    let request = live_scope();
    let principal = authenticated_principal(&depth, &request);
    let Err(denial) = admitted_capability_access(&depth, &principal, &request, 100) else {
        panic!("a chain beyond the installed depth must deny");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::DelegationDepthExceeded
    );
}

#[test]
fn resource_and_grantor_grantee_link_drift_deny_at_the_transition() {
    let resource = installed_delegated_capability_world();
    resource.authorization_time.script([time(100)]);
    replace_parent_resource(&resource);
    let request = live_scope();
    let principal = authenticated_principal(&resource, &request);
    let Err(denial) = admitted_capability_access(&resource, &principal, &request, 100) else {
        panic!("a parent bound to another resource must deny");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::DelegationRejected
    );

    let actors = installed_delegated_capability_world();
    actors.authorization_time.script([time(100)]);
    replace_child_grantor_with_grantee(&actors);
    let request = live_scope();
    let principal = authenticated_principal(&actors, &request);
    let Err(denial) = admitted_capability_access(&actors, &principal, &request, 100) else {
        panic!("a child grantor who is not the parent grantee must deny");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::DelegationRejected
    );
}

#[test]
fn exact_parent_policy_cannot_borrow_an_alternate_grant_path() {
    let world = installed_delegated_capability_world();
    world.authorization_time.script([time(100)]);
    replace_parent_grantor_with_child_grantee(&world);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    let Err(denial) = admitted_capability_access(&world, &principal, &request, 100) else {
        panic!("an alternate grant's allowing path must not authorize the exact parent");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing,
    );
}

#[test]
fn every_parent_currentness_drift_stales_retained_authority() {
    for drift in [
        ParentDrift::Revocation,
        ParentDrift::Expiry,
        ParentDrift::Workflow,
        ParentDrift::Resource,
        ParentDrift::DelegationGrantor,
        ParentDrift::Grantee,
        ParentDrift::PolicyPath,
        ParentDrift::Substitute,
    ] {
        let world = installed_delegated_capability_world();
        world.authorization_time.script([time(100), time(100)]);
        let request = live_scope();
        let principal = authenticated_principal(&world, &request);
        let access = admitted_capability_access(&world, &principal, &request, 100).unwrap();
        apply_parent_drift(&world, drift);
        let operation = world
            .application
            .installed_schema()
            .installed_operation(super::super::fixture::CapabilityTouchOperation::reference())
            .unwrap();
        let Err(denial) = world.application.authorize_capability_operation(
            access,
            &operation,
            Default::default(),
        ) else {
            panic!("retained authority must stale before a parent can be refreshed");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
            "drift {drift:?} did not invalidate the composite fact"
        );
    }
}

#[derive(Clone, Copy, Debug)]
enum ParentDrift {
    Revocation,
    Expiry,
    Workflow,
    Resource,
    DelegationGrantor,
    Grantee,
    PolicyPath,
    Substitute,
}

fn apply_parent_drift(world: &super::super::fixture::AuthorizationWorld, drift: ParentDrift) {
    match drift {
        ParentDrift::Revocation => update_grant_field(
            world,
            "capability-parent",
            field(world, CapabilityStatusField::reference()),
            super::super::fixture::CapabilityStatusBinding::encode(
                &super::super::fixture::CapabilityStatus::Revoked,
            )
            .unwrap(),
        ),
        ParentDrift::Expiry => update_grant_field(
            world,
            "capability-parent",
            field(world, CapabilityNotAfterField::reference()),
            U64ApplicationValueBinding::encode(&99_u64).unwrap(),
        ),
        ParentDrift::Workflow => update_grant_field(
            world,
            "capability-parent",
            field(world, CapabilityWorkflowField::reference()),
            StringApplicationValueBinding::encode(&"closed".to_owned()).unwrap(),
        ),
        ParentDrift::Resource => replace_parent_resource(world),
        ParentDrift::DelegationGrantor => replace_child_grantor_with_grantee(world),
        ParentDrift::Grantee => replace_parent_grantee(world),
        ParentDrift::PolicyPath => replace_parent_policy_path(world),
        ParentDrift::Substitute => replace_parent(
            world,
            "capability-child",
            "capability-parent",
            "capability-alternate",
        ),
    }
}

fn replace_parent_resource(world: &super::super::fixture::AuthorizationWorld) {
    let kind = relation_kind(world, CapabilityResource::reference().name());
    replace_relation_target(
        world,
        kind,
        grant(world, "capability-parent"),
        account(world, "account-1"),
        account(world, "account-2"),
        "parent-other-resource",
    );
}

fn replace_child_grantor_with_grantee(world: &super::super::fixture::AuthorizationWorld) {
    let child = grant(world, "capability-child");
    let grantor_kind = relation_kind(world, CapabilityGrantor::reference().name());
    let grantee_kind = relation_kind(world, CapabilityGrantee::reference().name());
    replace_relation_source(
        world,
        grantor_kind,
        relation_source(world, grantor_kind, child),
        relation_source(world, grantee_kind, child),
        child,
        "child-wrong-grantor",
    );
}

fn replace_parent_grantee(world: &super::super::fixture::AuthorizationWorld) {
    let child = grant(world, "capability-child");
    let parent = grant(world, "capability-parent");
    let grantee_kind = relation_kind(world, CapabilityGrantee::reference().name());
    replace_relation_source(
        world,
        grantee_kind,
        relation_source(world, grantee_kind, parent),
        relation_source(world, grantee_kind, child),
        parent,
        "parent-wrong-grantee",
    );
}

fn replace_parent_grantor_with_child_grantee(world: &super::super::fixture::AuthorizationWorld) {
    let child = grant(world, "capability-child");
    let parent = grant(world, "capability-parent");
    let grantor_kind = relation_kind(world, CapabilityGrantor::reference().name());
    let grantee_kind = relation_kind(world, CapabilityGrantee::reference().name());
    replace_relation_source(
        world,
        grantor_kind,
        relation_source(world, grantor_kind, parent),
        relation_source(world, grantee_kind, child),
        parent,
        "parent-wrong-grantor",
    );
}

fn replace_parent_policy_path(world: &super::super::fixture::AuthorizationWorld) {
    let parent = grant(world, "capability-parent");
    let grantor_kind = relation_kind(world, CapabilityGrantor::reference().name());
    let grantor = relation_source(world, grantor_kind, parent);
    replace_relation_kind(
        world,
        grantor_kind,
        relation_kind(world, CapabilityCustodian::reference().name()),
        grantor,
        parent,
        "parent-replacement-policy-path",
    );
}
