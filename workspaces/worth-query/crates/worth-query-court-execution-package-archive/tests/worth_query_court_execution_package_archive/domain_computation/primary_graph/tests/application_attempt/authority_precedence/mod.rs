//! Pins one authority order at every typed admission re-entry: runtime
//! ownership is refused before authentication lapse and before any read.

use std::time::Duration;
use std::time::SystemTime;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_required_binding::ProgramRequiredInput;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_required_binding::ProgramRequiredOperation;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ProgramRequiredMutationBinding;
use worth_query_execution::facade::domain_computation::MutationHandlerExecutionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyResolutionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationProjectionDenialKind;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

pub type Admission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    ProgramRequiredOperation,
    ProgramRequiredInput,
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account,
>;

pub const LIFETIME: Duration = Duration::from_secs(2);

/// Mints `count` admissions on `world` under one short-lived authentication.
pub fn expiring_admissions(
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
    count: usize,
) -> (Vec<Admission>, SystemTime, u64) {
    let external = world.authenticate("alice", LIFETIME, request);
    let expires_at = external.expires_at();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), world, "open", request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    let admissions = (0..count)
        .map(|_| {
            world
                .application
                .with_host_advancement(|phase| {
                    world
                        .selected_product(&(phase).test_host_request())
                        .authorize_operation(
                            &phase,
                            &principal,
                            &account,
                            &operation,
                            TypedMutationPreconditions::new(),
                            request,
                        )
                        .unwrap()
                })
                .expect("the fixture read owns its installed host call")
        })
        .collect::<Vec<_>>();
    (admissions, expires_at, *principal.principal_identity())
}

pub fn execute_on(
    world: &AuthorizationWorld,
    principal_identity: u64,
    admission: Admission,
) -> MutationHandlerExecutionDenial {
    let key = "precedence".to_owned();
    let input = ProgramRequiredInput::new("open");
    let identities = ApplicationMutationIdentities::<
        IdentityExecutionSchema,
        ProgramRequiredMutationBinding,
    >::encode(&key, &input)
    .expect("the request encodes");
    world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_mutation_handler::<ProgramRequiredMutationBinding>(
                    &phase,
                    &identities,
                    &principal_identity,
                    admission,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("a foreign admission never executes a target handler")
        })
        .expect("fixture owner admits its advancement")
}

pub fn refused(kind: AuthorizationRefusal) -> WorthQueryOperationAuthorizationDenialKind {
    WorthQueryOperationAuthorizationDenialKind::Refused(kind)
}

pub fn assert_refused_as_foreign_runtime(denial: MutationHandlerExecutionDenial) {
    let MutationHandlerExecutionDenial::Projection(projection) = denial else {
        panic!("a foreign admission is refused as foreign: {denial:?}");
    };
    assert_eq!(
        projection.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(refused(
            AuthorizationRefusal::ForeignRuntime
        ))
    );
}

#[test]
fn a_live_foreign_admission_is_refused_as_foreign_at_mutation_execution() {
    let source_world = installed_authorization_world(true);
    let target_world = installed_authorization_world(true);
    let request = live_scope();
    let (mut admissions, _, principal_identity) = expiring_admissions(&source_world, &request, 1);
    assert_refused_as_foreign_runtime(execute_on(
        &target_world,
        principal_identity,
        admissions.remove(0),
    ));
}

#[test]
fn a_foreign_and_expired_admission_is_refused_as_foreign_at_every_entry() {
    let source_world = installed_authorization_world(true);
    let target_world = installed_authorization_world(true);
    let request = live_scope();
    let (mut admissions, expires_at, principal_identity) =
        expiring_admissions(&source_world, &request, 3);
    for admission in &admissions {
        assert!(admission.validate_current_authority().is_ok());
    }
    let until_expiry = expires_at
        .duration_since(SystemTime::now())
        .unwrap_or_default();
    std::thread::sleep(until_expiry + Duration::from_millis(10));
    assert_eq!(
        admissions[0]
            .validate_current_authority()
            .unwrap_err()
            .kind(),
        refused(AuthorizationRefusal::ExpiredAuthentication)
    );

    let projection = target_world
        .application
        .with_host_advancement(|phase| {
            target_world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admissions[0],
                    |_, _| (),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("a foreign admission never projects")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        projection.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(refused(
            AuthorizationRefusal::ForeignRuntime
        ))
    );

    let resolution = target_world
        .application
        .with_host_advancement(|phase| {
            target_world
                .application
                .resolve_admitted_application_idempotency(
                    &phase,
                    &admissions[0],
                    idempotency(17, 17),
                )
                .expect_err("a foreign admission never resolves idempotency")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        resolution.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
    );

    let read = target_world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            target_world
                .application
                .begin_application_read_attempt(
                    &host_custody.borrowed_request_for_test(),
                    admissions.remove(0),
                )
                .err()
                .expect("a foreign admission never begins a read attempt")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        read.kind(),
        WorthQueryApplicationAttemptDenialKind::ForeignApplication
    );

    assert_refused_as_foreign_runtime(execute_on(
        &target_world,
        principal_identity,
        admissions.remove(0),
    ));
}
