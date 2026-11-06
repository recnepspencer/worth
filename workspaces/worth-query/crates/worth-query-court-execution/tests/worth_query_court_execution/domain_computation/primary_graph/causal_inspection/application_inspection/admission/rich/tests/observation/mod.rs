//! Genuine original acquisition for the private rich-body substitution test.
use worth_query_execution::facade::application_inspection::WorthQueryApplicationQueryObservation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityTouchInput;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityTouchOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::TouchAccountCapability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::DiagnosticParametersBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::shape::DiagnosticResult;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::DiagnosticResultBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ExternalMapping;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryFieldScope;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;

pub type Scope = ApplicationQueryFieldScope<AccountStatus>;

pub trait DiagnosticBinding:
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

pub type Capability = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    TouchAccountCapability,
    CapabilityTouchOperation,
    CapabilityTouchInput,
>;

pub fn capability(
    world: &AuthorizationWorld,
    external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
    request: &WorthQueryRequestScope,
) -> Capability {
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            admit_touch_account_capability(
                &phase.test_host_request_for_test(),
                world,
                &principal,
                request,
            )
            .unwrap()
        })
        .unwrap()
}

pub fn read<B: DiagnosticBinding>(
    world: &AuthorizationWorld,
    external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
    request: &WorthQueryRequestScope,
    status: &str,
    retain: bool,
) -> (WorthQueryApplicationQueryObservation, Vec<DiagnosticResult>) {
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request_for_test());
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
                    AccountStatus::at("AccountStatus"),
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
                &phase.test_host_request_for_test(),
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
