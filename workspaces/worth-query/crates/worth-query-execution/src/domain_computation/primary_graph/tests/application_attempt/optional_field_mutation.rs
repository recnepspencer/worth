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

#[test]
fn optional_fields_preserve_empty_zero_and_lawful_absence_through_query() {
    let world = super::super::fixture::installed_authorization_world(true);
    let request = live_scope();
    let principal = super::authenticated_principal(&world, &request);
    let account = super::resolved_account(&world, "unrelated", &request);

    assert_eq!(
        query(&world, &principal, "account-2", &request).note(),
        None
    );
    assert_eq!(
        query(&world, &principal, "account-2", &request).score(),
        None
    );

    let set = program(
        &world,
        &principal,
        &account,
        &request,
        Some(String::new()),
        Some(0),
    );
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(set, super::idempotency(71, 71)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let present = query(&world, &principal, "account-2", &request);
    assert_eq!(present.note(), Some(""));
    assert_eq!(present.score(), Some(0));

    let current = super::resolved_account(&world, "unrelated", &request);
    let clear = program(&world, &principal, &current, &request, None, None);
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(clear, super::idempotency(72, 72)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let absent = query(&world, &principal, "account-2", &request);
    assert_eq!(absent.note(), None);
    assert_eq!(absent.score(), None);
}

#[test]
fn ordinary_and_optional_writes_to_one_entity_commit_as_one_native_patch() {
    let world = super::super::fixture::installed_authorization_world(true);
    let request = live_scope();
    let principal = super::authenticated_principal(&world, &request);
    let account = super::resolved_account(&world, "unrelated", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(PatchAccountDraftOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .decision_field(projected, AccountNote::reference())
                .unwrap();
            reader
                .decision_field(projected, AccountScore::reference())
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
        .write_field(&account, AccountNote::reference(), "ordinary".to_owned())
        .unwrap();
    effects
        .write_optional_field(&account, AccountScore::reference(), Some(0))
        .unwrap();

    let outcome = world
        .application
        .compare_and_commit_application(effects.finish().unwrap(), super::idempotency(75, 75));
    assert!(matches!(
        outcome,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let result = query(&world, &principal, "account-2", &request);
    assert_eq!(result.note(), Some("ordinary"));
    assert_eq!(result.score(), Some(0));
}

#[test]
fn absent_field_decision_facts_stale_after_a_competing_presence_change() {
    let world = super::super::fixture::installed_authorization_world(true);
    let request = live_scope();
    let principal = super::authenticated_principal(&world, &request);
    let account = super::resolved_account(&world, "unrelated", &request);
    let winner = program(
        &world,
        &principal,
        &account,
        &request,
        Some("winner".to_owned()),
        Some(1),
    );
    let loser = program(
        &world,
        &principal,
        &account,
        &request,
        Some("loser".to_owned()),
        Some(2),
    );

    assert!(matches!(
        world
            .application
            .compare_and_commit_application(winner, super::idempotency(73, 73)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    super::assert_product_basis_stale(
        world
            .application
            .compare_and_commit_application(loser, super::idempotency(74, 74)),
        "a presence change after the losing product was selected",
    );
    let result = query(&world, &principal, "account-2", &request);
    assert_eq!(result.note(), Some("winner"));
    assert_eq!(result.score(), Some(1));
}

fn program(
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
        .selected_product()
        .authorize_operation(
            principal,
            account,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .decision_field(projected, AccountNote::reference())
                .unwrap();
            reader
                .decision_field(projected, AccountScore::reference())
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
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> OptionalAccountFieldResult {
    let scope = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            account.to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(OptionalAccountFieldQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(principal, &scope);
    let plan = world
        .selected_product()
        .admit_application_query(
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
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    result.rows()[0].clone()
}
