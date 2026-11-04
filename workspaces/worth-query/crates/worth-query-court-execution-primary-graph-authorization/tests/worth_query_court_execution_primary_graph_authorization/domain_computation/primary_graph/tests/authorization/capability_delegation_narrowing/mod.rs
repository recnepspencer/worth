use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityActionField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityAmountField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityDelegationLimitField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityDisclosureField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityNotAfterField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityNotBeforeField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityPurposeField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityRelated;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityWorkflowField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_delegated_capability_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::account;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::field;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::grant;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::relation_kind;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::replace_relation_target;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::update_grant_field;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_access;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;

#[derive(Clone, Copy, Debug)]
pub enum NarrowingAxis {
    Action,
    Purpose,
    Disclosure,
    RelatedRelationship,
    Magnitude,
    Workflow,
    ValidityStart,
    ValidityEnd,
    DownstreamDelegation,
}

#[test]
fn every_stored_narrowing_axis_is_enforced_by_production_admission() {
    for axis in [
        NarrowingAxis::Action,
        NarrowingAxis::Purpose,
        NarrowingAxis::Disclosure,
        NarrowingAxis::RelatedRelationship,
        NarrowingAxis::Magnitude,
        NarrowingAxis::Workflow,
        NarrowingAxis::ValidityStart,
        NarrowingAxis::ValidityEnd,
        NarrowingAxis::DownstreamDelegation,
    ] {
        let world = installed_delegated_capability_world();
        world.authorization_time.script([time(100)]);
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                break_narrowing_axis(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    axis,
                )
            })
            .expect("the fixture read owns its installed host call");
        let request = live_scope();
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                authenticated_principal(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");

        let Err(denial) = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                admitted_capability_access(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &principal,
                    &request,
                    100,
                )
            })
            .expect("the fixture read owns its installed host call")
        else {
            panic!("a widened delegated grant must mint no access authority");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::DelegationRejected
            ),
            "axis {axis:?} escaped the installed narrowing transition"
        );
    }
}

pub fn break_narrowing_axis(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    axis: NarrowingAxis,
) {
    if matches!(axis, NarrowingAxis::RelatedRelationship) {
        replace_parent_related_relationship(phase_custody, world);
        return;
    }
    let (field, value, grant) = match axis {
        NarrowingAxis::Action => (
            field(world, CapabilityActionField::reference()),
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityActionBinding::encode(
                &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction::Inspect,
            )
            .unwrap(),
            "capability-parent",
        ),
        NarrowingAxis::Purpose => (
            field(world, CapabilityPurposeField::reference()),
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityPurposeBinding::encode(
                &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose::Audit,
            )
            .unwrap(),
            "capability-parent",
        ),
        NarrowingAxis::Disclosure => (
            field(world, CapabilityDisclosureField::reference()),
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding::encode(
                &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure::PrivateLabel,
            )
            .unwrap(),
            "capability-parent",
        ),
        NarrowingAxis::Magnitude => (
            field(world, CapabilityAmountField::reference()),
            U64ApplicationValueBinding::encode(&76_u64).unwrap(),
            "capability-child",
        ),
        NarrowingAxis::Workflow => (
            field(world, CapabilityWorkflowField::reference()),
            StringApplicationValueBinding::encode(&"closed".to_owned()).unwrap(),
            "capability-parent",
        ),
        NarrowingAxis::ValidityStart => (
            field(world, CapabilityNotBeforeField::reference()),
            U64ApplicationValueBinding::encode(&89_u64).unwrap(),
            "capability-child",
        ),
        NarrowingAxis::ValidityEnd => (
            field(world, CapabilityNotAfterField::reference()),
            U64ApplicationValueBinding::encode(&111_u64).unwrap(),
            "capability-child",
        ),
        NarrowingAxis::DownstreamDelegation => (
            field(world, CapabilityDelegationLimitField::reference()),
            U64ApplicationValueBinding::encode(&2_u64).unwrap(),
            "capability-child",
        ),
        NarrowingAxis::RelatedRelationship => unreachable!("handled above"),
    };
    update_grant_field(phase_custody, world, grant, field, value);
}

pub fn replace_parent_related_relationship(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) {
    let parent = grant(
        &(phase_custody).test_host_request_for_test(),
        &phase_custody.borrowed_request_for_test(),
        world,
        "capability-parent",
    );
    replace_relation_target(
        &(phase_custody).test_host_request_for_test(),
        &phase_custody.borrowed_request_for_test(),
        world,
        relation_kind(world, CapabilityRelated::reference().name()),
        parent,
        account(
            &(phase_custody).test_host_request_for_test(),
            &phase_custody.borrowed_request_for_test(),
            world,
            "account-2",
        ),
        account(
            &(phase_custody).test_host_request_for_test(),
            &phase_custody.borrowed_request_for_test(),
            world,
            "account-1",
        ),
        "parent-wider-related-relationship",
    );
}
