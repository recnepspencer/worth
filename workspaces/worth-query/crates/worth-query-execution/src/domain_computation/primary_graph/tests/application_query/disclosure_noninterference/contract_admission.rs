use std::num::NonZeroUsize;
use std::time::{Duration, UNIX_EPOCH};

use worth_foundational::facade::{
    AspectMask, CanonicalFieldPath, DiagnosticMask, FieldKey, ProjectionMask,
};
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureSelector;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::super::super::fixture::{
    admit_touch_account_capability, forbidden_live_identity_parameters,
    installed_capability_world_with_label, live_scope, status_parameter, Account, AccountIdentity,
    AccountLabel, AuthorizationWorld, ForbiddenHiddenOrderingQuery, ForbiddenInfluenceQuery,
    ForbiddenLiveScopeIdentityQuery, ForbiddenLiveTargetIdentityQuery, IdentityExecutionSchema,
    IncompleteDisclosureQuery, Principal, ResultRulePredicateQuery,
};
use crate::domain_computation::authorization::application_disclosure::contract::admit_field_mask_categories;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrincipalResolutionMode,
};

struct AdmissionContext {
    world: AuthorizationWorld,
    request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    principal: WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
}

#[test]
fn contract_incompatible_projection_mask_denies_before_result_construction() {
    let context = admission_context();
    let field = AccountLabel::reference();
    let field_key = FieldKey::new(field.field()).unwrap();
    let selector = ApplicationQueryDisclosureSelector::InternalField {
        entity: field.entity().to_owned(),
        aspect: field.aspect().to_owned(),
        field: field.field().to_owned(),
        projection_mask: AspectMask::<ProjectionMask>::new([CanonicalFieldPath::single(
            FieldKey::new("not-an-installed-field").unwrap(),
        )]),
        diagnostic_mask: AspectMask::<DiagnosticMask>::new([CanonicalFieldPath::single(field_key)]),
    };
    let layout = context
        .world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout();

    assert!(
        admit_field_mask_categories(
            layout,
            &selector,
            (field.entity(), field.aspect(), field.field()),
        )
        .is_err(),
        "a Query selector cannot widen the Foundational AspectContract field universe"
    );
}

#[path = "contract_admission/predicate_access.rs"]
mod predicate_access;

#[test]
fn incomplete_result_contract_denies_before_result_construction() {
    let context = admission_context();
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(IncompleteDisclosureQuery::reference())
        .unwrap();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let capability = admit_touch_account_capability(
                &(host_custody.phase_for_application(&context.world.application))
                    .test_host_request(),
                &context.world,
                &context.principal,
                &context.request,
            )
            .unwrap();
            let access =
                WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
            let denial = context
                .world
                .selected_product(
                    &(host_custody.phase_for_application(&context.world.application))
                        .test_host_request(),
                )
                .admit_governed_application_query(
                    &host_custody.phase_for_application(&context.world.application),
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::<IncompleteDisclosureQuery>::new(),
                    controls(&context.request),
                )
                .err()
                .expect("every result slot requires an explicit disclosure rule");
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn forbidden_predicate_influence_denies_before_result_construction() {
    let context = admission_context();
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(ForbiddenInfluenceQuery::reference())
        .unwrap();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let capability = admit_touch_account_capability(
                &(host_custody.phase_for_application(&context.world.application))
                    .test_host_request(),
                &context.world,
                &context.principal,
                &context.request,
            )
            .unwrap();
            let access =
                WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
            let denial = context
                .world
                .selected_product(
                    &(host_custody.phase_for_application(&context.world.application))
                        .test_host_request(),
                )
                .admit_governed_application_query(
                    &host_custody.phase_for_application(&context.world.application),
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_owned())
                        .expect("fixture query parameter must encode"),
                    controls(&context.request),
                )
                .err()
                .expect("predicate reads require row-presence and count influence");
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn forbidden_ordering_influence_denies_before_result_construction() {
    let context = admission_context();
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(ForbiddenHiddenOrderingQuery::reference())
        .unwrap();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let capability = admit_touch_account_capability(
                &(host_custody.phase_for_application(&context.world.application))
                    .test_host_request(),
                &context.world,
                &context.principal,
                &context.request,
            )
            .unwrap();
            let access =
                WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
            let denial = context
                .world
                .selected_product(
                    &(host_custody.phase_for_application(&context.world.application))
                        .test_host_request(),
                )
                .admit_governed_application_query(
                    &host_custody.phase_for_application(&context.world.application),
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::<ForbiddenHiddenOrderingQuery>::new(),
                    controls(&context.request),
                )
                .err()
                .expect("a protected ordering value requires ordering influence");
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn forbidden_live_scope_influence_denies_before_result_construction() {
    let context = admission_context();
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(ForbiddenLiveScopeIdentityQuery::reference())
        .unwrap();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let capability = admit_touch_account_capability(
                &(host_custody.phase_for_application(&context.world.application))
                    .test_host_request(),
                &context.world,
                &context.principal,
                &context.request,
            )
            .unwrap();
            let access =
                WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
            let denial = context
                .world
                .selected_product(
                    &(host_custody.phase_for_application(&context.world.application))
                        .test_host_request(),
                )
                .admit_governed_application_query(
                    &host_custody.phase_for_application(&context.world.application),
                    &query,
                    &access,
                    capability,
                    forbidden_live_identity_parameters::<ForbiddenLiveScopeIdentityQuery>(),
                    controls(&context.request),
                )
                .err()
                .expect("a protected live scope identity requires live-membership influence");
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn forbidden_live_target_influence_denies_before_result_construction() {
    let context = admission_context();
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(ForbiddenLiveTargetIdentityQuery::reference())
        .unwrap();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let capability = admit_touch_account_capability(
                &(host_custody.phase_for_application(&context.world.application))
                    .test_host_request(),
                &context.world,
                &context.principal,
                &context.request,
            )
            .unwrap();
            let access =
                WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
            let denial = context
                .world
                .selected_product(
                    &(host_custody.phase_for_application(&context.world.application))
                        .test_host_request(),
                )
                .admit_governed_application_query(
                    &host_custody.phase_for_application(&context.world.application),
                    &query,
                    &access,
                    capability,
                    forbidden_live_identity_parameters::<ForbiddenLiveTargetIdentityQuery>(),
                    controls(&context.request),
                )
                .err()
                .expect("a protected live target identity requires live-membership influence");
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
            );
        })
        .expect("the fixture block owns its installed host call")
}

fn admission_context() -> AdmissionContext {
    let world = installed_capability_world_with_label("private");
    world.authorization_time.script([
        UNIX_EPOCH + Duration::from_secs(100),
        UNIX_EPOCH + Duration::from_secs(100),
    ]);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);

    let (principal, account) = world
        .application
        .with_host_advancement(|phase| {
            let host_request = phase.test_host_request();
            let phase_custody = &host_request;
            let borrowed_execution = phase
                .execution_request_for(world.application.product_runtime())
                .expect("the fixture lends its installed phase");
            let execution = &borrowed_execution;
            let principal = world
                .application
                .select_product_branch(
                    execution,
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase_custody.phase_for_application(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = world
                .application
                .select_product_branch(
                    execution,
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase_custody.phase_for_application(&world.application),
                    AccountIdentity::reference(),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();

            (principal, account)
        })
        .expect("the read-context fixture owns its identity call");
    AdmissionContext {
        world,
        request,
        principal,
        account,
    }
}

fn controls(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> crate::domain_computation::primary_graph::WorthQueryProductQueryControls<'_> {
    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(256).unwrap(),
        request,
    )
}
