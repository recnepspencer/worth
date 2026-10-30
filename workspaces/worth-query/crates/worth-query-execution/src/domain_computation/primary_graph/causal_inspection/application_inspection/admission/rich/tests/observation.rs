//! Genuine original acquisition for the private rich-body substitution test.
use crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationQueryObservation;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        admit_touch_account_capability,
        capability::{CapabilityTouchInput, CapabilityTouchOperation, TouchAccountCapability},
        diagnostic_comparison::{
            DiagnosticParametersBinding, DiagnosticResult, DiagnosticResultBinding,
        },
        status_parameter, Account, AccountPolicy, AccountStatus, AccountSummaryParameters,
        AuthorizationWorld, ExternalMapping, IdentityBinding, IdentityExecutionSchema, Principal,
    },
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
};
use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryFieldScope, ApplicationQueryParameterSet,
    },
    application_schema::{NoApplicationUnit, ReadWrite, U64ApplicationValueBinding},
};

type Scope = ApplicationQueryFieldScope<
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    AccountStatus,
    String,
    ReadWrite,
    NoApplicationUnit,
>;

pub(in super::super) trait DiagnosticBinding:
    ApplicationQueryBinding<
    IdentityExecutionSchema,
    Input = AccountSummaryParameters,
    InputBinding = DiagnosticParametersBinding,
    ParameterBinding = DiagnosticParametersBinding,
    ResultBinding = DiagnosticResultBinding,
    ScopeBinding = Scope,
    PrincipalBinding = IdentityBinding,
    Mapping = ExternalMapping,
    Principal = Principal,
    PrincipalIdentity = u64,
    PrincipalIdentityBinding = U64ApplicationValueBinding,
>
{
}
impl<B> DiagnosticBinding for B where
    B: ApplicationQueryBinding<
        IdentityExecutionSchema,
        Input = AccountSummaryParameters,
        InputBinding = DiagnosticParametersBinding,
        ParameterBinding = DiagnosticParametersBinding,
        ResultBinding = DiagnosticResultBinding,
        ScopeBinding = Scope,
        PrincipalBinding = IdentityBinding,
        Mapping = ExternalMapping,
        Principal = Principal,
        PrincipalIdentity = u64,
        PrincipalIdentityBinding = U64ApplicationValueBinding,
    >
{
}

pub(super) type Capability = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    TouchAccountCapability,
    CapabilityTouchOperation,
    CapabilityTouchInput,
>;

pub(in super::super) fn capability(
    world: &AuthorizationWorld,
    external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
    request: &WorthQueryRequestScope,
) -> Capability {
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            admit_touch_account_capability(&phase.test_host_request(), world, &principal, request)
                .unwrap()
        })
        .unwrap()
}

pub(in super::super) fn read<B: DiagnosticBinding>(
    world: &AuthorizationWorld,
    external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
    request: &WorthQueryRequestScope,
    status: &str,
    retain: bool,
) -> (WorthQueryApplicationQueryObservation, Vec<DiagnosticResult>) {
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let scope = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let query = world
                .application
                .installed_schema()
                .installed_query_binding::<B>()
                .unwrap()
                .into_query();
            let capability = admit_touch_account_capability(
                &phase.test_host_request(),
                world,
                &principal,
                request,
            )
            .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
            let plan = selected
                .admit_governed_application_query(
                    &phase,
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), status.to_owned())
                        .unwrap(),
                    WorthQueryProductQueryControls::new(std::num::NonZeroUsize::MIN, request),
                )
                .unwrap();
            let plan = if retain {
                plan.retain_projection_for_inspection()
            } else {
                plan
            };
            let result = world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap();
            let rows = result.rows().to_vec();
            let original = result
                .into_admitted_disclosed()
                .into_output_demand_source()
                .retain_inspection_observation()
                .unwrap();
            (original, rows)
        })
        .unwrap()
}
