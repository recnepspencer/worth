use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
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
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");

    let access = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_access(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
                100,
            )
            .expect("the narrowed current child and exact parent must admit")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(access.authorization_decision_fact_count(), 2);
    assert_eq!(access.relational_counters().paths_evaluated, 20);
    assert!(access.signal_dependency_count() > 0);
    let work = access.admission_canonical_work();
    // The one canonical derivation is the governed input the admission encoded.
    assert_eq!(work.basis_preparations(), 0);
    assert_eq!(work.digest_derivations(), 1);
    assert_eq!(work.canonical_encoded_bytes(), 289);
    assert_eq!(work.sha256_input_bytes(), 377);
    assert_eq!(work.sha256_compression_blocks(), 7);
    assert_eq!(work.digest_text_materializations(), 0);
}

#[path = "capability_delegation/width_and_cycle_attacks_are_typed_denials_before_access_authorit.rs"]
mod width_and_cycle_attacks_are_typed_denials_before_access_authorit;

#[path = "capability_delegation/resource_and_grantor_grantee_link_drift_deny_at_the_transition.rs"]
mod resource_and_grantor_grantee_link_drift_deny_at_the_transition;

#[test]
fn exact_parent_policy_cannot_borrow_an_alternate_grant_path() {
    let world = installed_delegated_capability_world();
    world.authorization_time.script([time(100)]);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            replace_parent_grantor_with_child_grantee(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
            )
        })
        .expect("the fixture read owns its installed host call");
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_access(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
                100,
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("an alternate grant's allowing path must not authorize the exact parent");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityAuthorizationMissing
        ),
    );
}

#[path = "capability_delegation/every_parent_currentness_drift_stales_retained_authority.rs"]
mod every_parent_currentness_drift_stales_retained_authority;

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

fn apply_parent_drift(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
    drift: ParentDrift,
) {
    match drift {
        ParentDrift::Revocation => update_grant_field(
            phase_custody,
            world,
            "capability-parent",
            field(world, CapabilityStatusField::reference()),
            super::super::fixture::CapabilityStatusBinding::encode(
                &super::super::fixture::CapabilityStatus::Revoked,
            )
            .unwrap(),
        ),
        ParentDrift::Expiry => update_grant_field(
            phase_custody,
            world,
            "capability-parent",
            field(world, CapabilityNotAfterField::reference()),
            U64ApplicationValueBinding::encode(&99_u64).unwrap(),
        ),
        ParentDrift::Workflow => update_grant_field(
            phase_custody,
            world,
            "capability-parent",
            field(world, CapabilityWorkflowField::reference()),
            StringApplicationValueBinding::encode(&"closed".to_owned()).unwrap(),
        ),
        ParentDrift::Resource => {
            replace_parent_resource(&(phase_custody).test_host_request(), world)
        }
        ParentDrift::DelegationGrantor => replace_child_grantor_with_grantee(
            &(phase_custody).test_host_request(),
            execution,
            world,
        ),
        ParentDrift::Grantee => replace_parent_grantee(phase_custody, execution, world),
        ParentDrift::PolicyPath => replace_parent_policy_path(phase_custody, execution, world),
        ParentDrift::Substitute => replace_parent(
            phase_custody,
            world,
            "capability-child",
            "capability-parent",
            "capability-alternate",
        ),
    }
}

fn replace_parent_resource(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &super::super::fixture::AuthorizationWorld,
) {
    let kind = relation_kind(world, CapabilityResource::reference().name());
    replace_relation_target(
        phase_custody,
        &phase_custody.borrowed_request_for_test(),
        world,
        kind,
        grant(
            phase_custody,
            &phase_custody.borrowed_request_for_test(),
            world,
            "capability-parent",
        ),
        account(
            phase_custody,
            &phase_custody.borrowed_request_for_test(),
            world,
            "account-1",
        ),
        account(
            phase_custody,
            &phase_custody.borrowed_request_for_test(),
            world,
            "account-2",
        ),
        "parent-other-resource",
    );
}

fn replace_child_grantor_with_grantee(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
) {
    let child = grant(phase_custody, execution, world, "capability-child");
    let grantor_kind = relation_kind(world, CapabilityGrantor::reference().name());
    let grantee_kind = relation_kind(world, CapabilityGrantee::reference().name());
    replace_relation_source(
        phase_custody,
        execution,
        world,
        grantor_kind,
        relation_source(phase_custody, execution, world, grantor_kind, child),
        relation_source(phase_custody, execution, world, grantee_kind, child),
        child,
        "child-wrong-grantor",
    );
}

fn replace_parent_grantee(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
) {
    let child = grant(
        &(phase_custody).test_host_request(),
        execution,
        world,
        "capability-child",
    );
    let parent = grant(
        &(phase_custody).test_host_request(),
        execution,
        world,
        "capability-parent",
    );
    let grantee_kind = relation_kind(world, CapabilityGrantee::reference().name());
    replace_relation_source(
        &(phase_custody).test_host_request(),
        execution,
        world,
        grantee_kind,
        relation_source(
            &(phase_custody).test_host_request(),
            execution,
            world,
            grantee_kind,
            parent,
        ),
        relation_source(
            &(phase_custody).test_host_request(),
            execution,
            world,
            grantee_kind,
            child,
        ),
        parent,
        "parent-wrong-grantee",
    );
}

fn replace_parent_grantor_with_child_grantee(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
) {
    let child = grant(
        &(phase_custody).test_host_request(),
        execution,
        world,
        "capability-child",
    );
    let parent = grant(
        &(phase_custody).test_host_request(),
        execution,
        world,
        "capability-parent",
    );
    let grantor_kind = relation_kind(world, CapabilityGrantor::reference().name());
    let grantee_kind = relation_kind(world, CapabilityGrantee::reference().name());
    replace_relation_source(
        &(phase_custody).test_host_request(),
        execution,
        world,
        grantor_kind,
        relation_source(
            &(phase_custody).test_host_request(),
            execution,
            world,
            grantor_kind,
            parent,
        ),
        relation_source(
            &(phase_custody).test_host_request(),
            execution,
            world,
            grantee_kind,
            child,
        ),
        parent,
        "parent-wrong-grantor",
    );
}

fn replace_parent_policy_path(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
) {
    let parent = grant(
        &(phase_custody).test_host_request(),
        execution,
        world,
        "capability-parent",
    );
    let grantor_kind = relation_kind(world, CapabilityGrantor::reference().name());
    let grantor = relation_source(
        &(phase_custody).test_host_request(),
        execution,
        world,
        grantor_kind,
        parent,
    );
    replace_relation_kind(
        phase_custody,
        execution,
        world,
        grantor_kind,
        relation_kind(world, CapabilityCustodian::reference().name()),
        grantor,
        parent,
        "parent-replacement-policy-path",
    );
}
