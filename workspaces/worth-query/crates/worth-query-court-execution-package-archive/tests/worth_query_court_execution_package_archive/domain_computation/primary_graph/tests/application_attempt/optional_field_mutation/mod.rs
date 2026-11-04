pub use std::num::NonZeroUsize;

pub use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
pub use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountNote;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountScore;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::OptionalAccountFieldQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::optional_account_field_query::OptionalAccountFieldResult;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::PatchAccountDraftInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PatchAccountDraftOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
pub use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub type Program = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    PatchAccountDraftOperation,
    PatchAccountDraftInput,
    Account,
>;

#[test]
fn ordinary_and_optional_writes_to_one_entity_commit_as_one_native_patch() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account(&(phase).test_host_request_for_test(), &world, "unrelated", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(PatchAccountDraftOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (_, projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, projected| {
                        reader
                            .decision_field(projected, AccountNote::reference())
                            .unwrap();
                        reader
                            .decision_field(projected, AccountScore::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("the fixture read owns its installed host call");
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(&account, AccountNote::reference(), "ordinary".to_owned())
        .unwrap();
    effects
        .write_optional_field(&account, AccountScore::reference(), Some(0))
        .unwrap();
    effects
        .write_field(&account, AccountNote::reference(), "final".to_owned())
        .unwrap();
    effects
        .write_optional_field(&account, AccountScore::reference(), Some(5))
        .unwrap();

    let outcome = world.application.compare_and_commit_application(
        effects.finish().unwrap(),
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency(75, 75),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "optional-field commit refused: {outcome:?}"
    );
    let result = world
        .application
        .with_host_advancement(|phase| {
            query(
                &phase,
                &(phase).borrowed_request_for_test(),
                &world,
                &principal,
                "account-2",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(result.note(), Some("final"));
    assert_eq!(result.score(), Some(5));
}

#[test]
fn absent_field_decision_facts_stale_after_a_competing_presence_change() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account(&(phase).test_host_request_for_test(), &world, "unrelated", &request)
        })
        .expect("the fixture read owns its installed host call");
    let winner = world
        .application
        .with_host_advancement(|phase| {
            program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                Some("winner".to_owned()),
                Some(1),
            )
        })
        .expect("the fixture read owns its installed host call");
    let loser = world
        .application
        .with_host_advancement(|phase| {
            program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                Some("loser".to_owned()),
                Some(2),
            )
        })
        .expect("the fixture read owns its installed host call");

    assert!(matches!(
        world.application.compare_and_commit_application(
            winner,
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency(73, 73),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::decision_currentness::assert_changed_decision(
        world.application.compare_and_commit_application(
            loser,
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency(74, 74),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ),
        "a presence change after the losing product was selected",
    );
    let result = world
        .application
        .with_host_advancement(|phase| {
            query(
                &phase,
                &(phase).borrowed_request_for_test(),
                &world,
                &principal,
                "account-2",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(result.note(), Some("winner"));
    assert_eq!(result.score(), Some(1));
}

pub fn program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    note: Option<String>,
    score: Option<u64>,
) -> Program {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(PatchAccountDraftOperation::reference())
        .unwrap();
    let admission = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .authorize_operation(
            phase_custody,
            principal,
            account,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, projected| {
                reader
                    .decision_field(projected, AccountNote::reference())
                    .unwrap();
                reader
                    .decision_field(projected, AccountScore::reference())
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
    let account = effects.existing_entity(account).unwrap();
    effects
        .write_optional_field(&account, AccountNote::reference(), note)
        .unwrap();
    effects
        .write_optional_field(&account, AccountScore::reference(), score)
        .unwrap();
    effects.finish().unwrap()
}

pub fn query(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> OptionalAccountFieldResult {
    let scope = world
        .application
        .select_product_branch_for_test(
            execution,
            world.application.product_runtime().default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            phase_custody,
            AccountIdentity::reference(),
            account.to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(OptionalAccountFieldQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(principal, &scope);
    let plan = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .admit_application_query(
            phase_custody,
            &query,
            &access,
            ApplicationQueryParameterSet::<OptionalAccountFieldQuery>::new(),
            worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                request,
            ),
        )
        .unwrap();
    let result = world
        .application
        .execute_application_query_one_shot(phase_custody, plan)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    result.rows()[0].clone()
}

#[path = "optional_fields_preserve_empty_zero_and_lawful_absence_through_q/mod.rs"]
pub mod optional_fields_preserve_empty_zero_and_lawful_absence_through_q;
