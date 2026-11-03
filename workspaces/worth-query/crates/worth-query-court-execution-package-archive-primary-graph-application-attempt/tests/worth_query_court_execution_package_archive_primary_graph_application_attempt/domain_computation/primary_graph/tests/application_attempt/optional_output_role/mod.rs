//! An at-most-one output role through the installed mutation handler lane:
//! the handler may leave it unbound or bind it once, both commit and read back
//! as `None` and `Some`, a second binding is refused, and an exactly-one role
//! left unbound is still missing. Naming the role with another cardinality
//! fails to compile; only its erased form, as a readmitted role takes, reaches
//! Query, and the committed correspondence refuses it.

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_execution::equality_type_id::EqualityTypeId as TypeId;

pub use worth_query_execution::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::role_use::OutputRoleUse;
pub use worth_query_execution::facade::domain_computation::HandlerResult;
pub use worth_query_execution::facade::domain_computation::WorthQueryCompletedMutationCandidate;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::optional_output_binding::OptionalCompanion;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::optional_output_binding::OptionalOutputInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::OptionalOutputMutationBinding;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::OptionalOutputOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::optional_output_binding::OptionalOutputPlan;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::optional_output_binding::OptionalOutputs;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::optional_output_binding::OptionalSubject;
pub use worth_query_execution::facade::domain_computation::MutationHandlerExecutionDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
pub use worth_query_execution::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationOutputProjectionDenial;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleCardinality;
pub use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
pub use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

#[test]
fn an_unbound_at_most_one_role_commits_without_that_output() {
    let committed = committed_outputs(OptionalOutputPlan::RequiredOnly);
    let outputs = committed.outputs_of::<OptionalOutputs>().unwrap();

    assert!(outputs.entity::<OptionalSubject>().is_ok());
    assert!(
        outputs
            .entity::<OptionalCompanion>()
            .expect("an absent optional output is a value, not a denial")
            .is_none(),
        "the unbound optional role reads as None"
    );
    assert_eq!(
        committed.workflow_content_identity(),
        committed_outputs(OptionalOutputPlan::RequiredOnly).workflow_content_identity(),
        "an absent optional output has one stable canonical form"
    );
}

#[test]
fn a_bound_at_most_one_role_commits_with_that_output() {
    let committed = committed_outputs(OptionalOutputPlan::RequiredAndOptional);
    let outputs = committed.outputs_of::<OptionalOutputs>().unwrap();

    assert_eq!(
        outputs
            .entity::<OptionalCompanion>()
            .unwrap()
            .map(|companion| companion.entity_id()),
        Some(outputs.entity::<OptionalSubject>().unwrap().entity_id())
    );
    assert_ne!(
        committed.workflow_content_identity(),
        committed_outputs(OptionalOutputPlan::RequiredOnly).workflow_content_identity(),
        "a present optional output is distinct from an absent one"
    );
}

#[test]
fn a_second_binding_of_an_at_most_one_role_is_refused() {
    let Err(MutationHandlerExecutionDenial::Handler(denial)) =
        executed(OptionalOutputPlan::RequiredAndOptionalTwice)
    else {
        panic!("a second binding of an at-most-one role must be refused");
    };
    let denial = denial
        .downcast::<WorthQueryApplicationAttemptDenial>()
        .expect("the refusal is the attempt's typed denial");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::DuplicateOutputRole
    );
    assert_eq!(denial.subject(), "companion");
}

#[test]
fn an_optional_role_read_as_exactly_one_is_refused() {
    let companion_as_required = OutputRoleUse {
        name: "companion".to_owned(),
        posture: ApplicationMutationOutputPosture::Preserve,
        cardinality: ApplicationMutationOutputRoleCardinality::ExactlyOne,
        entity_name:
            <Account as ApplicationEntityMarkerIdentity<IdentityExecutionSchema>>::IDENTIFIER,
        entity_type: TypeId::of::<Account>(),
        contract_type: TypeId::of::<OptionalOutputs>(),
    };
    for plan in [
        OptionalOutputPlan::RequiredOnly,
        OptionalOutputPlan::RequiredAndOptional,
    ] {
        assert_eq!(
            committed_outputs(plan)
                .bound_entity(&companion_as_required)
                .err(),
            Some(WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch),
            "{plan:?}"
        );
    }
}

#[test]
fn an_unbound_exactly_one_role_is_still_missing() {
    let Err(MutationHandlerExecutionDenial::Attempt(denial)) =
        executed(OptionalOutputPlan::OptionalOnly)
    else {
        panic!("a completed candidate must bind every exactly-one role");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::MissingOutputRole
    );
    assert_eq!(denial.subject(), "subject");
}

pub type Execution = Result<
    HandlerResult<
        WorthQueryCompletedMutationCandidate<
            IdentityExecutionSchema,
            OptionalOutputMutationBinding,
        >,
        OptionalOutputInput,
    >,
    MutationHandlerExecutionDenial,
>;

pub fn committed_outputs(plan: OptionalOutputPlan) -> WorthQueryApplicationOutputCorrespondence {
    let world = installed_authorization_world(true);
    let (execution, idempotency) = execute(&world, plan);
    let Ok(HandlerResult::Completed(completed)) = execution else {
        panic!("the handler completes its candidate");
    };
    let (program, _) = completed.into_parts();
    let WorthQueryApplicationCommitOutcome::Committed(committed) =
        world.application.compare_and_commit_application(
            program,
            idempotency,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the completed candidate commits");
    };
    committed.output_correspondence().clone()
}

pub fn executed(plan: OptionalOutputPlan) -> Execution {
    execute(&installed_authorization_world(true), plan).0
}

pub fn execute(
    world: &AuthorizationWorld,
    plan: OptionalOutputPlan,
) -> (Execution, WorthQueryApplicationIdempotencyBinding) {
    execute_with_key(world, plan, "optional-output")
}

pub fn execute_with_key(
    world: &AuthorizationWorld,
    plan: OptionalOutputPlan,
    key: &str,
) -> (Execution, WorthQueryApplicationIdempotencyBinding) {
    let (report, key) = execute_report_with_key(world, plan, key);
    (report.into_outcome(), key)
}

pub fn execute_report_with_key(
    world: &AuthorizationWorld,
    plan: OptionalOutputPlan,
    key: &str,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryMutationHandlerExecutionReport<
        WorthQueryCompletedMutationCandidate<
            IdentityExecutionSchema,
            OptionalOutputMutationBinding,
        >,
        OptionalOutputInput,
    >,
    WorthQueryApplicationIdempotencyBinding,
) {
    execute_report_for_input(
        world,
        OptionalOutputInput {
            status: "open".to_owned(),
            plan,
        },
        key,
    )
}

pub fn execute_report_for_input(
    world: &AuthorizationWorld,
    input: OptionalOutputInput,
    key: &str,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryMutationHandlerExecutionReport<
        WorthQueryCompletedMutationCandidate<
            IdentityExecutionSchema,
            OptionalOutputMutationBinding,
        >,
        OptionalOutputInput,
    >,
    WorthQueryApplicationIdempotencyBinding,
) {
    let request = live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let custody = phase.test_host_request();
            let principal = authenticated_principal(&custody, world, &request);
            let account = resolved_account(&custody, world, &input.status, &request);
            let operation = world
                .application
                .installed_schema()
                .installed_operation(OptionalOutputOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(&custody)
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .unwrap();
            let key = key.to_owned();
            let identities = ApplicationMutationIdentities::<
                IdentityExecutionSchema,
                OptionalOutputMutationBinding,
            >::encode(&key, &input)
            .expect("the request encodes");
            let idempotency =
                WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
            let execution = world
                .application
                .execute_mutation_handler_report::<OptionalOutputMutationBinding>(
                    &phase,
                    &identities,
                    principal.principal_identity(),
                    admission,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                );
            (execution, idempotency)
        })
        .expect("the fixture policy admits its handler call")
}

#[path = "handler_work/mod.rs"]
pub mod handler_work;
#[path = "indexed_selection/mod.rs"]
pub mod indexed_selection;
#[path = "source_disclosure/mod.rs"]
pub mod source_disclosure;
