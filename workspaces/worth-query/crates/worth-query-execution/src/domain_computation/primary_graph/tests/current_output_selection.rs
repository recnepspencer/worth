use std::time::Duration;

use super::fixture::{
    installed_authorization_world, live_scope, Account, AccountStatus, Principal,
    TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
    WorthQueryCurrentOutputDenialKind, WorthQueryCurrentOutputSelection,
    WorthQueryOperationProjectionDenialKind, WorthQueryPrincipalResolutionMode,
    WorthQueryProducerApplicability, WorthQueryProducerOutputFamily,
};

struct TestCurrentOutputFamily;

impl WorthQueryProducerOutputFamily<super::fixture::IdentityExecutionSchema>
    for TestCurrentOutputFamily
{
    type Source = super::fixture::TestAccountSourceBinding;
    type Entity = Account;

    const IDENTITY: &'static str = "worth.query.test.current-output-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[];

    fn profile_kind(_: &super::fixture::AccountSummaryResult) -> &'static str {
        "account"
    }
}

#[test]
fn current_output_rejects_a_foreign_producer_identity_with_its_typed_cause() {
    let world = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let foreign_account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
            admitted_touch_parts(&phase, &(phase).borrowed_request_for_test(), &world)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
            admitted_touch_parts(&phase, &(phase).borrowed_request_for_test(), &world)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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

mod current_output_reports_an_obsolete_source_after_authoritative_re;

fn retire_open_account(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &super::fixture::AuthorizationWorld,
) {
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
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(&account).unwrap();
    effects
        .delete_entity(Account::reference(), &account)
        .unwrap();
    let program = effects.finish().unwrap();
    let outcome = world.application.compare_and_commit_application(
        program,
        WorthQueryApplicationIdempotencyBinding::new([201; 32], [202; 32]),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    if !matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)) {
        panic!("authoritative retirement must commit: {outcome:?}");
    }
}

fn admitted_touch_parts(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::fixture::AuthorizationWorld,
) -> (
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        super::fixture::IdentityExecutionSchema,
        Principal,
        u64,
    >,
    crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
        super::fixture::IdentityExecutionSchema,
        Account,
    >,
    worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
        super::fixture::IdentityExecutionSchema,
        TouchAccountOperation,
        super::fixture::TouchAccountInput,
    >,
) {
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

mod current_output_source_adjacency_tracks_actual_anchor_retirement;
