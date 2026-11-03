pub use std::time::Duration;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
pub use worth_query_execution::facade::domain_computation::WorthQueryCurrentOutputDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryCurrentOutputSelection;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationProjectionDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability;
pub use worth_query_execution::facade::application_contribution::WorthQueryProducerOutputFamily;

pub struct TestCurrentOutputFamily;

impl WorthQueryProducerOutputFamily<worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema>
    for TestCurrentOutputFamily
{
    type Source = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::current_output_source::TestAccountSourceBinding;
    type Entity = Account;

    const IDENTITY: &'static str = "worth.query.test.current-output-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[];

    fn profile_kind(_: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryResult) -> &'static str {
        "account"
    }
}

#[test]
fn current_output_rejects_a_foreign_producer_identity_with_its_typed_cause() {
    let world = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let foreign_account = foreign
        .application
        .with_host_advancement(|phase| {
            foreign
                .invariant
                .project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_entity(AccountStatus::reference(), "open".to_owned())
                            .unwrap()
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .expect("the fixture read owns its installed host call");
    let (request, principal, scope, operation) = world
        .application
        .with_host_advancement(|phase| {
            admitted_touch_parts(&phase, &(phase).borrowed_request_for_test(), &world)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &scope,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, _| {
                        reader.current_output::<TestCurrentOutputFamily, Account>(&foreign_account)
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let Err(denial) = completed.output() else {
        panic!("a foreign producer identity must not select an output");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryCurrentOutputDenialKind::ForeignIdentity
    );
}

#[test]
fn current_output_exhaustion_denies_the_admitted_projection() {
    let world = installed_authorization_world(true);
    let (request, principal, scope, operation) = world
        .application
        .with_host_advancement(|phase| {
            admitted_touch_parts(&phase, &(phase).borrowed_request_for_test(), &world)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &scope,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let projected = world
        .application
        .with_host_advancement(|phase| {
            world.invariant.project_admitted_operation(
                &phase,
                &admission,
                |reader, root| {
                    for _ in 0..31 {
                        reader.field(root, AccountStatus::reference()).unwrap();
                    }
                    reader.current_output::<TestCurrentOutputFamily, Account>(root)
                },
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
        })
        .expect("the fixture read owns its installed host call");
    let Err(denial) = projected else {
        panic!("selector work beyond the admitted budget must deny the projection");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationProjectionDenialKind::WorkBudgetExceeded
    );
}

pub fn retire_open_account(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) {
    let program = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase;
            let (request, principal, account, operation) = admitted_touch_parts(
                phase_custody,
                &(phase_custody).borrowed_request_for_test(),
                world,
            );
            let admission = world
                .selected_product(&(phase_custody).test_host_request())
                .authorize_operation(
                    phase_custody,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    phase_custody,
                    &admission,
                    |reader, account| {
                        reader
                            .require_decision_entity(account, Account::reference())
                            .unwrap();
                        reader
                            .require_decision_field(account, AccountStatus::reference())
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
                .delete_entity(Account::reference(), &account)
                .unwrap();

            effects.finish().unwrap()
        })
        .expect("the retirement preparation owns its host call");
    let outcome = world.application.compare_and_commit_application(
        program,
        WorthQueryApplicationIdempotencyBinding::new([201; 32], [202; 32]),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    if !matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)) {
        panic!("authoritative retirement must commit: {outcome:?}");
    }
}

pub fn admitted_touch_parts(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> (
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        Principal,
        u64,
    >,
    worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        Account,
    >,
    worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        TouchAccountOperation,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountInput,
    >,
){
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap()
        .resolve_authenticated_principal(
            phase_custody,
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap()
        .resolve_entity(
            phase_custody,
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    (request, principal, scope, operation)
}

#[path = "counted_read_stops/mod.rs"]
pub mod counted_read_stops;
#[path = "current_output_reports_an_obsolete_source_after_authoritative_re/mod.rs"]
pub mod current_output_reports_an_obsolete_source_after_authoritative_re;
#[path = "current_output_source_adjacency_tracks_actual_anchor_retirement/mod.rs"]
pub mod current_output_source_adjacency_tracks_actual_anchor_retirement;
