pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityConflictingBeneficiary;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityPriorActor;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityRequestActor;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::ComposedCapabilityTouchOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::ComposedTouchAccountCapability;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_composed_capability_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_seed::composition::CapabilityCompositionScenario;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityTouchInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_composition_mutation::add_action_policy_relation;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_composition_mutation::add_policy_relation;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_composition_mutation::remove_policy_relation;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
pub use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationAuthorizationExplanationCause;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;

pub type ComposedAccess = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    ComposedTouchAccountCapability,
    ComposedCapabilityTouchOperation,
    CapabilityTouchInput,
>;
pub type ComposedProgram = WorthQueryApplicationEffectProgram<
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
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
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

#[test]
fn unrelated_actor_records_do_not_poison_the_selected_transition() {
    let world =
        installed_composed_capability_world(CapabilityCompositionScenario::UnrelatedActorRecords);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
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
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
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
            let host_custody = phase.test_host_request_for_test();
            remove_policy_relation(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner::reference(),
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
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
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
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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

pub fn admit_composed_access(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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
        .selected_product(&(phase_custody).test_host_request_for_test())
        .admit_capability_access(
            phase_custody,
            principal,
            &capability,
            composed_input(),
            request,
        )
}

pub fn assert_stale_at_operation(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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

pub fn composed_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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
    let account = resolved_account(&(phase_custody).test_host_request_for_test(), world, "open", request);
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(&account, AccountLabel::reference(), "composed".to_owned())
        .unwrap();
    effects.finish().unwrap()
}

pub fn composed_input() -> CapabilityTouchInput {
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

#[path = "every_installed_combination_predicate_denies_independently_and_t/mod.rs"]
pub mod every_installed_combination_predicate_denies_independently_and_t;
#[path = "unrelated_published_actor_drift_preserves_the_prepared_product_m/mod.rs"]
pub mod unrelated_published_actor_drift_preserves_the_prepared_product_m;
