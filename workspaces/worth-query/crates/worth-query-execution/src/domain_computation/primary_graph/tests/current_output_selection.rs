use std::time::Duration;

use super::fixture::{
    installed_authorization_world, live_scope, Account, AccountStatus, Principal,
    TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
    WorthQueryCurrentOutputDenialKind, WorthQueryCurrentOutputRole,
    WorthQueryCurrentOutputSelection, WorthQueryOperationProjectionDenialKind,
    WorthQueryPrincipalResolutionMode, WorthQueryProducerApplicability,
    WorthQueryProducerOutputFamily,
};

struct TestCurrentOutputFamily;

impl WorthQueryProducerOutputFamily<super::fixture::IdentityExecutionSchema>
    for TestCurrentOutputFamily
{
    type Source = super::fixture::TestAccountSourceBinding;

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
    let foreign_account = foreign
        .invariant
        .project(|reader| {
            reader
                .resolve_entity(AccountStatus::reference(), "open".to_owned())
                .unwrap()
        })
        .unwrap()
        .into_parts()
        .0;
    let (request, principal, scope, operation) = admitted_touch_parts(&world);
    let admission = world
        .selected_product()
        .authorize_operation(&principal, &scope, &operation, Default::default(), &request)
        .unwrap();

    let completed = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            reader.current_output::<TestCurrentOutputFamily, Account, Account>(
                &foreign_account,
                WorthQueryCurrentOutputRole::new("anchor"),
            )
        })
        .unwrap();
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
    let (request, principal, scope, operation) = admitted_touch_parts(&world);
    let admission = world
        .selected_product()
        .authorize_operation(&principal, &scope, &operation, Default::default(), &request)
        .unwrap();

    let projected = world
        .invariant
        .project_admitted_operation(&admission, |reader, root| {
            for _ in 0..31 {
                reader.field(root, AccountStatus::reference()).unwrap();
            }
            reader.current_output::<TestCurrentOutputFamily, Account, Account>(
                root,
                WorthQueryCurrentOutputRole::new("anchor"),
            )
        });
    let Err(denial) = projected else {
        panic!("selector work beyond the admitted budget must deny the projection");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationProjectionDenialKind::WorkBudgetExceeded
    );
}

#[test]
fn current_output_reports_an_obsolete_source_after_authoritative_retirement() {
    let world = installed_authorization_world(true);
    let obsolete = world
        .invariant
        .project(|reader| {
            reader
                .resolve_entity(AccountStatus::reference(), "open".to_owned())
                .unwrap()
        })
        .unwrap()
        .into_parts()
        .0;
    retire_open_account(&world);

    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let surviving_scope = selected
        .resolve_entity(
            AccountStatus::reference(),
            "unrelated".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = selected
        .authorize_operation(
            &principal,
            &surviving_scope,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();

    let completed = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            reader.current_output::<TestCurrentOutputFamily, Account, Account>(
                &obsolete,
                WorthQueryCurrentOutputRole::new("anchor"),
            )
        })
        .unwrap();
    assert!(matches!(
        completed.output(),
        Ok(WorthQueryCurrentOutputSelection::ObsoleteSource)
    ));
}

fn retire_open_account(world: &super::fixture::AuthorizationWorld) {
    let (request, principal, account, operation) = admitted_touch_parts(world);
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, account| {
            reader
                .require_decision_entity(account, Account::reference())
                .unwrap();
            reader
                .require_decision_field(account, AccountStatus::reference())
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
        .delete_entity(Account::reference(), &account)
        .unwrap();
    let program = effects.finish().unwrap();
    let outcome = world.application.compare_and_commit_application(
        program,
        WorthQueryApplicationIdempotencyBinding::new([201; 32], [202; 32]),
    );
    if !matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)) {
        panic!("authoritative retirement must commit: {outcome:?}");
    }
}

fn admitted_touch_parts(
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
        .select_product_branch(world.application.product_runtime().default_branch())
        .unwrap()
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .unwrap()
        .resolve_entity(
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
