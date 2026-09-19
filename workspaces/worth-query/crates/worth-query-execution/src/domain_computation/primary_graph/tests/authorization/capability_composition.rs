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
    let principal = authenticated_principal(&world, &request);

    let access = admit_composed_access(&world, &principal, &request).unwrap();

    assert_eq!(access.authorization_decision_fact_count(), 2);
    assert_eq!(access.relational_counters().paths_evaluated, 9);
    assert_eq!(access.signal_dependency_count(), 31);
}

#[test]
fn every_installed_combination_predicate_denies_independently_and_together() {
    for (scenario, expected_causes, expected_explanation) in [
        (
            CapabilityCompositionScenario::MissingAssignment,
            vec![WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing],
            WorthQueryApplicationAuthorizationExplanationCause::MissingCapability,
        ),
        (
            CapabilityCompositionScenario::ExplicitDeny,
            vec![WorthQueryOperationAuthorizationDenialKind::ExplicitDenyRuleMatched],
            WorthQueryApplicationAuthorizationExplanationCause::ExplicitPolicyDenial,
        ),
        (
            CapabilityCompositionScenario::ConflictingBeneficiary,
            vec![WorthQueryOperationAuthorizationDenialKind::ConflictRuleMatched],
            WorthQueryApplicationAuthorizationExplanationCause::Conflict,
        ),
        (
            CapabilityCompositionScenario::RequestActor,
            vec![WorthQueryOperationAuthorizationDenialKind::SeparationOfDutyRuleMatched],
            WorthQueryApplicationAuthorizationExplanationCause::SeparationOfDuty,
        ),
        (
            CapabilityCompositionScenario::PriorActor,
            vec![WorthQueryOperationAuthorizationDenialKind::DistinctActorRuleMatched],
            WorthQueryApplicationAuthorizationExplanationCause::SeparationOfDuty,
        ),
        (
            CapabilityCompositionScenario::AccumulatedProhibitions,
            vec![
                WorthQueryOperationAuthorizationDenialKind::ExplicitDenyRuleMatched,
                WorthQueryOperationAuthorizationDenialKind::ConflictRuleMatched,
                WorthQueryOperationAuthorizationDenialKind::SeparationOfDutyRuleMatched,
                WorthQueryOperationAuthorizationDenialKind::DistinctActorRuleMatched,
            ],
            WorthQueryApplicationAuthorizationExplanationCause::ExplicitPolicyDenial,
        ),
    ] {
        let world = installed_composed_capability_world(scenario);
        world.authorization_time.script([time(100)]);
        let request = live_scope();
        let principal = authenticated_principal(&world, &request);

        let denial = admit_composed_access(&world, &principal, &request)
            .err()
            .unwrap_or_else(|| panic!("{scenario:?} must deny at initial access admission"));

        assert!(denial.identity().is_some(), "{scenario:?}");
        assert_eq!(denial.kind(), expected_causes[0], "{scenario:?}");
        assert_eq!(denial.causes(), expected_causes, "{scenario:?}");
        assert_eq!(
            denial.explanation_cause(),
            Some(expected_explanation),
            "{scenario:?}"
        );
    }
}

#[test]
fn unrelated_actor_records_do_not_poison_the_selected_transition() {
    let world =
        installed_composed_capability_world(CapabilityCompositionScenario::UnrelatedActorRecords);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    admit_composed_access(&world, &principal, &request).unwrap();
}

#[test]
fn required_assignment_loss_denies_at_operation_progression() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
    world.authorization_time.script([time(100), time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let access = admit_composed_access(&world, &principal, &request).unwrap();

    remove_policy_relation(&world, super::super::fixture::AccountOwner::reference());

    assert_stale_at_operation(&world, access);
}

#[test]
fn new_conflict_denies_at_operation_progression() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
    world.authorization_time.script([time(100), time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let access = admit_composed_access(&world, &principal, &request).unwrap();

    add_policy_relation(
        &world,
        CapabilityConflictingBeneficiary::reference(),
        "late-conflicting-beneficiary",
    );

    assert_stale_at_operation(&world, access);
}

#[test]
fn new_prior_actor_denies_final_commit_before_effect_authority() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
    world.authorization_time.script(vec![time(100); 16]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let program = composed_program(&world, &principal, &request);

    add_action_policy_relation(
        &world,
        CapabilityPriorActor::reference(),
        "late-prior-actor",
        "selected-prior",
    );

    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(program, idempotency(75, 75))
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

#[test]
fn unrelated_published_actor_drift_stales_the_selected_product_mutation() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
    world.authorization_time.script(vec![time(100); 16]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let program = composed_program(&world, &principal, &request);

    add_action_policy_relation(
        &world,
        CapabilityRequestActor::reference(),
        "late-unrelated-request-actor",
        "other-request",
    );
    add_action_policy_relation(
        &world,
        CapabilityPriorActor::reference(),
        "late-unrelated-prior-actor",
        "other-prior",
    );

    super::super::application_attempt::assert_product_basis_stale(
        world
            .application
            .compare_and_commit_application(program, idempotency(76, 76)),
        "the mutation bound to the older selected product",
    );
}

fn admit_composed_access(
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
    world.selected_product().admit_capability_access(
        principal,
        &capability,
        composed_input(),
        request,
    )
}

fn assert_stale_at_operation(world: &AuthorizationWorld, access: ComposedAccess) {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ComposedCapabilityTouchOperation::reference())
        .unwrap();
    let denial = world
        .application
        .authorize_capability_operation(access, &operation, Default::default())
        .err()
        .expect("composition drift must deny at operation progression");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
    );
}

fn composed_program(
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> ComposedProgram {
    let access = admit_composed_access(world, principal, request).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ComposedCapabilityTouchOperation::reference())
        .unwrap();
    let admission = world
        .application
        .authorize_capability_operation(access, &operation, Default::default())
        .unwrap();
    let account = resolved_account(world, "open", request);
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountLabel::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
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
        governed_input_identity: Default::default(),
    }
}
