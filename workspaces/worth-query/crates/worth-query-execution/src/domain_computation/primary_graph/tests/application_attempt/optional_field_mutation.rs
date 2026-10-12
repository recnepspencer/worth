use std::num::NonZeroUsize;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use super::super::fixture::{
    live_scope, Account, AccountIdentity, AccountNote, AccountScore, AuthorizationWorld,
    IdentityExecutionSchema, OptionalAccountFieldQuery, OptionalAccountFieldResult,
    PatchAccountDraftInput, PatchAccountDraftOperation, Principal,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationQueryAccessContext,
    WorthQueryAuthenticatedPrincipal, WorthQueryPrincipalResolutionMode,
};

type Program = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    PatchAccountDraftOperation,
    PatchAccountDraftInput,
    Account,
>;

#[path = "optional_field_mutation/optional_fields_preserve_empty_zero_and_lawful_absence_through_q.rs"]
mod optional_fields_preserve_empty_zero_and_lawful_absence_through_q;

#[test]
fn ordinary_and_optional_writes_to_one_entity_commit_as_one_native_patch() {
    let world = super::super::fixture::installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            super::authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            super::resolved_account(&(phase).test_host_request(), &world, "unrelated", &request)
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
                .selected_product(&(phase).test_host_request())
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
    let reads = world
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
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = world
        .application
        .with_host_advancement(|phase| {
            reads
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
        super::idempotency(75, 75),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(matches!(
        outcome,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
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
    let world = super::super::fixture::installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            super::authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            super::resolved_account(&(phase).test_host_request(), &world, "unrelated", &request)
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
            super::idempotency(73, 73),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    super::assert_changed_decision(
        world.application.compare_and_commit_application(
            loser,
            super::idempotency(74, 74),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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

fn program(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        .selected_product(&(phase_custody).test_host_request())
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
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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

fn query(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> OptionalAccountFieldResult {
    let scope = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
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
        .selected_product(&(phase_custody).test_host_request())
        .admit_application_query(
            phase_custody,
            &query,
            &access,
            ApplicationQueryParameterSet::<OptionalAccountFieldQuery>::new(),
            crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(256).unwrap(),
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
