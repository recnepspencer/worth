use super::super::application_attempt::{authenticated_principal, idempotency, resolved_account};
use super::super::fixture::capability::{
    CapabilityConflictingBeneficiary, CapabilityPriorActor, CapabilityRequestActor,
    ComposedCapabilityTouchOperation, ComposedTouchAccountCapability,
};
use super::super::fixture::{
    installed_composed_capability_world, live_scope, Account, AccountLabel, AuthorizationWorld,
    CapabilityAction, CapabilityCompositionScenario, CapabilityDisclosure, CapabilityPurpose,
    CapabilityTouchInput, IdentityExecutionSchema, Principal,
};
use super::capability_composition_mutation::{
    add_action_policy_relation, add_policy_relation, remove_policy_relation,
};
use super::capability_progression::time;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationCapabilityAccess,
    WorthQueryApplicationAuthorizationExplanationCause, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryAuthenticatedPrincipal,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

type ComposedAccess = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    ComposedTouchAccountCapability,
    ComposedCapabilityTouchOperation,
    CapabilityTouchInput,
>;
type ComposedProgram = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ComposedCapabilityTouchOperation,
    CapabilityTouchInput,
    Account,
>;

#[test]
fn lawful_combination_is_one_decision_over_presence_and_absence_facts() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
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
            admit_composed_access(&phase, &world, &principal, &request).unwrap()
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(access.authorization_decision_fact_count(), 2);
    assert_eq!(access.relational_counters().paths_evaluated, 9);
    assert_eq!(access.signal_dependency_count(), 31);
}

#[path = "capability_composition/every_installed_combination_predicate_denies_independently_and_t.rs"]
mod every_installed_combination_predicate_denies_independently_and_t;

#[test]
fn unrelated_actor_records_do_not_poison_the_selected_transition() {
    let world =
        installed_composed_capability_world(CapabilityCompositionScenario::UnrelatedActorRecords);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            admit_composed_access(&phase, &world, &principal, &request).unwrap()
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn required_assignment_loss_denies_at_operation_progression() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
    world.authorization_time.script([time(100), time(100)]);
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
            admit_composed_access(&phase, &world, &principal, &request).unwrap()
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            remove_policy_relation(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                super::super::fixture::AccountOwner::reference(),
            )
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| assert_stale_at_operation(&phase, &world, access))
        .expect("the fixture read owns its installed host call");
}

#[test]
fn new_conflict_denies_at_operation_progression() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
    world.authorization_time.script([time(100), time(100)]);
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
            admit_composed_access(&phase, &world, &principal, &request).unwrap()
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            add_policy_relation(
                &phase,
                &world,
                CapabilityConflictingBeneficiary::reference(),
                "late-conflicting-beneficiary",
            )
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| assert_stale_at_operation(&phase, &world, access))
        .expect("the fixture read owns its installed host call");
}

#[test]
fn new_prior_actor_denies_final_commit_before_effect_authority() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
    world.authorization_time.script(vec![time(100); 16]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| composed_program(&phase, &world, &principal, &request))
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            add_action_policy_relation(
                &phase,
                &world,
                CapabilityPriorActor::reference(),
                "late-prior-actor",
                "selected-prior",
            )
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            program,
            idempotency(75, 75),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("a late prior actor must deny before effect authority");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected,
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::DecisionReadSet,
    );
}

#[path = "capability_composition/unrelated_published_actor_drift_preserves_the_prepared_product_m.rs"]
mod unrelated_published_actor_drift_preserves_the_prepared_product_m;

fn admit_composed_access(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<ComposedAccess, WorthQueryOperationAuthorizationDenial> {
    let capability = world
        .application
        .installed_schema()
        .capability(
            ComposedTouchAccountCapability::reference(),
            ComposedCapabilityTouchOperation::reference(),
        )
        .unwrap();
    world
        .selected_product(&(phase_custody).test_host_request())
        .admit_capability_access(
            phase_custody,
            principal,
            &capability,
            composed_input(),
            request,
        )
}

fn assert_stale_at_operation(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    access: ComposedAccess,
) {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ComposedCapabilityTouchOperation::reference())
        .unwrap();
    let denial = world
        .application
        .authorize_capability_operation(phase_custody, access, &operation, Default::default())
        .err()
        .expect("composition drift must deny at operation progression");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization
        ),
    );
}

fn composed_program(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> ComposedProgram {
    let access = admit_composed_access(phase_custody, world, principal, request).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ComposedCapabilityTouchOperation::reference())
        .unwrap();
    let admission = world
        .application
        .authorize_capability_operation(phase_custody, access, &operation, Default::default())
        .unwrap();
    let account = resolved_account(&(phase_custody).test_host_request(), world, "open", request);
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, projected| {
                reader
                    .require_decision_field(projected, AccountLabel::reference())
                    .unwrap();
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            phase_custody,
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            phase_custody,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(&account, AccountLabel::reference(), "composed".to_owned())
        .unwrap();
    effects.finish().unwrap()
}

fn composed_input() -> CapabilityTouchInput {
    CapabilityTouchInput {
        account: "account-1".to_owned(),
        action: CapabilityAction::Touch,
        purpose: CapabilityPurpose::AccountMaintenance,
        disclosure: CapabilityDisclosure::AccountActivity,
        related_account: "account-2".to_owned(),
        amount: 50,
        caller_time: 100,
        request_record: "selected-request".to_owned(),
        prior_record: "selected-prior".to_owned(),
    }
}
