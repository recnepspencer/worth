pub use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityCustodian;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityGrantee;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityGrantor;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityNotAfterField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityResource;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityStatusField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityWorkflowField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_delegated_capability_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::account;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::add_parent;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::field;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::grant;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::relation_kind;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::relation_source;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::replace_parent;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::replace_relation_kind;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::replace_relation_source;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::replace_relation_target;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::update_grant_field;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_access;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;

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

#[derive(Clone, Copy, Debug)]
pub enum ParentDrift {
    Revocation,
    Expiry,
    Workflow,
    Resource,
    DelegationGrantor,
    Grantee,
    PolicyPath,
    Substitute,
}

pub fn apply_parent_drift(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    drift: ParentDrift,
) {
    match drift {
        ParentDrift::Revocation => update_grant_field(
            phase_custody,
            world,
            "capability-parent",
            field(world, CapabilityStatusField::reference()),
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityStatusBinding::encode(
                &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatus::Revoked,
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

pub fn replace_parent_resource(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
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

pub fn replace_child_grantor_with_grantee(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
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

pub fn replace_parent_grantee(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
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

pub fn replace_parent_grantor_with_child_grantee(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
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

pub fn replace_parent_policy_path(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
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

#[path = "every_parent_currentness_drift_stales_retained_authority/mod.rs"]
pub mod every_parent_currentness_drift_stales_retained_authority;
#[path = "resource_and_grantor_grantee_link_drift_deny_at_the_transition/mod.rs"]
pub mod resource_and_grantor_grantee_link_drift_deny_at_the_transition;
#[path = "width_and_cycle_attacks_are_typed_denials_before_access_authorit/mod.rs"]
pub mod width_and_cycle_attacks_are_typed_denials_before_access_authorit;
