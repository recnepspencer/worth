//! Forgery court for the erased admission core behind a typed brand. Every
//! forged or altered core is refused by an existing exact kind, in today's
//! precedence slot, before any graph, provider or request work.
//!
//! Mounted from the application-attempt tests, whose installed worlds and
//! entries it drives; the doors it uses live beside the brand.

use std::time::Duration;

use super::{
    authenticated_principal, idempotency, installed_authorization_world, live_scope,
    resolved_account, ProgramRequiredInput, ProgramRequiredOperation, TouchAccountOperation,
    WorthQueryPrincipalResolutionMode,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::fact_verification_observation::fact_verification_on_this_thread_for_test as fact_work;
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, AuthorizationWorld, IdentityExecutionSchema, Principal, ProgramRequiredMutationBinding,
};
use crate::domain_computation::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationEntityIdentity,
    WorthQueryApplicationIdempotencyResolutionDenialKind, WorthQueryAuthenticatedPrincipal,
    WorthQueryMutationHandlerWork, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryOperationProjectionDenialKind,
};
use crate::facade::primary_graph::installed_source_reads_on_this_thread_for_test as reads;
use crate::test_read_stops::{
    charged_prefixes_for_test, consult_count_for_test, observe_consults_for_test,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities,
};

type Admission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    ProgramRequiredOperation,
    ProgramRequiredInput,
    Account,
>;
type AuthenticatedPrincipal =
    WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>;
type AccountIdentity = WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>;

macro_rules! admit {
    ($world:expr, $principal:expr, $account:expr, $operation:ty, $request:expr) => {{
        let operation = $world
            .application
            .installed_schema()
            .installed_operation(<$operation>::reference())
            .unwrap();
        $world
            .application
            .with_host_advancement(|phase| {
                $world
                    .selected_product(&(phase).test_host_request())
                    .authorize_operation(
                        &phase,
                        $principal,
                        $account,
                        &operation,
                        Default::default(),
                        $request,
                    )
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call")
    }};
}

fn party(
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
) -> (AuthenticatedPrincipal, AccountIdentity) {
    world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            (
                authenticated_principal(&custody, world, request),
                resolved_account(&custody, world, "open", request),
            )
        })
        .expect("the fixture read owns its installed host call")
}

/// Everything one mutation execution did before it returned.
struct Execution {
    denial: Option<MutationHandlerExecutionDenial>,
    work: WorthQueryMutationHandlerWork,
    source_reads: u64,
    source_fact_admissions: u64,
    request_consults: usize,
    request_charges: usize,
}

impl Execution {
    fn did_nothing(&self) -> bool {
        self.work == WorthQueryMutationHandlerWork::NotStarted
            && self.source_reads == 0
            && self.source_fact_admissions == 0
            && self.request_consults == 0
            && self.request_charges == 0
    }
}

fn execute(world: &AuthorizationWorld, principal: u64, admission: Admission) -> Execution {
    let key = "forgery".to_owned();
    let input = ProgramRequiredInput::new("open");
    let identities = ApplicationMutationIdentities::<
        IdentityExecutionSchema,
        ProgramRequiredMutationBinding,
    >::encode(&key, &input)
    .expect("the request encodes");
    world
        .application
        .with_host_advancement(|phase| {
            let _observation =
                crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
            fact_work();
            let before = reads();
            let consults = observe_consults_for_test(|_| {});
            let (outcome, work) = world
                .application
                .execute_mutation_handler_report::<ProgramRequiredMutationBinding>(
                    &phase,
                    &identities,
                    &principal,
                    admission,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .into_parts();
            Execution {
                denial: outcome.err(),
                work,
                source_reads: reads() - before,
                source_fact_admissions: fact_work().source_admission_visits,
                request_consults: consult_count_for_test(&consults),
                request_charges: charged_prefixes_for_test(&consults).len(),
            }
        })
        .expect("fixture owner admits its advancement")
}

fn refused(kind: AuthorizationRefusal) -> WorthQueryOperationAuthorizationDenialKind {
    WorthQueryOperationAuthorizationDenialKind::Refused(kind)
}

fn assert_projection_refused(
    denial: Option<MutationHandlerExecutionDenial>,
    kind: AuthorizationRefusal,
) {
    let Some(MutationHandlerExecutionDenial::Projection(projection)) = denial else {
        panic!("a forged core is refused as {kind:?}: {denial:?}");
    };
    assert_eq!(
        projection.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(refused(kind))
    );
}

#[test]
fn a_core_rewrapped_as_another_operation_is_stale_before_any_work() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let (principal, account) = party(&world, &request);
    let identity = *principal.principal_identity();

    let control = execute(
        &world,
        identity,
        admit!(
            world,
            &principal,
            &account,
            ProgramRequiredOperation,
            &request
        ),
    );
    assert!(
        matches!(
            control.work,
            WorthQueryMutationHandlerWork::Captured(work) if work.handler_contacted()
        ),
        "the owned admission reaches its handler: {:?}",
        control.denial
    );
    assert!(
        control.request_consults > 0,
        "the same door consults its request"
    );

    let brand = admit!(
        world,
        &principal,
        &account,
        ProgramRequiredOperation,
        &request
    );
    let donor = admit!(world, &principal, &account, TouchAccountOperation, &request);
    let forged = execute(&world, identity, brand.forge_with_core_of_for_test(donor));
    assert!(forged.did_nothing());
    assert_projection_refused(forged.denial, AuthorizationRefusal::StaleInstalledOperation);
}

#[test]
fn a_core_from_another_runtime_is_foreign_under_either_brand() {
    let source = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let target = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let (source_principal, source_account) = party(&source, &request);
    let (target_principal, target_account) = party(&target, &request);
    let identity = *target_principal.principal_identity();
    let source_admission = || {
        admit!(
            source,
            &source_principal,
            &source_account,
            ProgramRequiredOperation,
            &request
        )
    };
    let target_admission = || {
        admit!(
            target,
            &target_principal,
            &target_account,
            ProgramRequiredOperation,
            &request
        )
    };

    // The target's brand around the source's core: the core is foreign.
    let foreign_core = target_admission().forge_with_core_of_for_test(source_admission());
    let execution = execute(&target, identity, foreign_core);
    assert!(execution.did_nothing());
    assert_projection_refused(execution.denial, AuthorizationRefusal::ForeignRuntime);

    // The source's brand around the target's own core: only the tag knows.
    let foreign_brand = source_admission().forge_with_core_of_for_test(target_admission());
    let execution = execute(&target, identity, foreign_brand);
    assert!(execution.did_nothing());
    assert_projection_refused(execution.denial, AuthorizationRefusal::ForeignRuntime);
}

#[test]
fn a_flipped_seal_byte_is_stale_at_mutation_and_foreign_at_idempotency() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let (principal, account) = party(&world, &request);
    let mut forged = admit!(
        world,
        &principal,
        &account,
        ProgramRequiredOperation,
        &request
    );
    forged.flip_core_seal_byte_for_test();
    let resolution = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .resolve_admitted_application_idempotency(&phase, &forged, idempotency(3, 3))
                .expect_err("a forged core never resolves idempotency")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        resolution.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
    );
    let execution = execute(&world, *principal.principal_identity(), forged);
    assert!(execution.did_nothing());
    assert_projection_refused(
        execution.denial,
        AuthorizationRefusal::StaleInstalledOperation,
    );
}

#[test]
fn a_successor_core_from_another_admission_is_refused_at_rewrap() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let (principal, account) = party(&world, &request);
    let predecessor = admit!(
        world,
        &principal,
        &account,
        ProgramRequiredOperation,
        &request
    );
    let other = admit!(
        world,
        &principal,
        &account,
        ProgramRequiredOperation,
        &request
    );
    let denial = predecessor
        .rewrap_successor_for_test(other)
        .err()
        .expect("an admission never continues as another one");
    assert_eq!(
        denial.kind(),
        refused(AuthorizationRefusal::InconsistentDecision)
    );
}

#[test]
fn a_brand_the_installed_binding_does_not_name_never_reaches_its_handler() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let (principal, account) = party(&world, &request);
    let identity = *principal.principal_identity();

    let mut input = admit!(
        world,
        &principal,
        &account,
        ProgramRequiredOperation,
        &request
    );
    input.forge_tag_input_type_for_test::<u8>();
    let execution = execute(&world, identity, input);
    assert!(execution.did_nothing());
    assert!(matches!(
        execution.denial,
        Some(MutationHandlerExecutionDenial::InputNotAdmitted)
    ));

    let mut operation = admit!(
        world,
        &principal,
        &account,
        ProgramRequiredOperation,
        &request
    );
    operation.forge_tag_operation_type_for_test::<TouchAccountOperation>();
    let execution = execute(&world, identity, operation);
    assert!(execution.did_nothing());
    assert_projection_refused(
        execution.denial,
        AuthorizationRefusal::StaleInstalledOperation,
    );
}

#[path = "erased_core_forgery_tests/allocation.rs"]
mod allocation;
#[path = "erased_core_forgery_tests/decision_entry.rs"]
mod decision_entry;
#[path = "erased_core_forgery_tests/precedence.rs"]
mod precedence;

#[path = "erased_core_forgery_tests/commit_entries.rs"]
mod commit_entries;
#[path = "erased_core_forgery_tests/installed_seal.rs"]
mod installed_seal;
#[path = "erased_core_forgery_tests/recovery_entries.rs"]
mod recovery_entries;
#[path = "erased_core_forgery_tests/workflow_entry.rs"]
mod workflow_entry;
