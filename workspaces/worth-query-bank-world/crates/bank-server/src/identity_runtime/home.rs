//! Bank's command attachment borrows the full generic Home's real program owner.
use super::*;
use crate::BankApplication;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticationAdapterAdmission;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

/// Build Bank's full static installation without attaching workflow execution.
/// The generic Home alone owns open/readmission/close/reclaim; seeds apply only
/// to a fresh Home through its existing first-creation initializer.
pub fn bank_application_home(
    location: WorthQueryApplicationHomeLocation,
    seed: BankWorldSeed,
) -> Result<
    WorthQueryApplicationHomeRequest<'static, BankSchema, BankApplication>,
    BankIdentityRuntimeBuildError,
> {
    use crate::application_definition::{
        validated_bank_application, validated_bank_application_p1, validated_bank_application_p2,
    };
    use worth_query_host::facade::application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationProgramRoster,
    };
    let (principals, world) = prepare_world(seed)?;
    let program = validated_bank_application()
        .map_err(BankIdentityRuntimeBuildError::ApplicationProgramValidation)?;
    let successor = validated_bank_application_p1()
        .map_err(BankIdentityRuntimeBuildError::ApplicationProgramValidation)?;
    let successor_p2 = validated_bank_application_p2()
        .map_err(BankIdentityRuntimeBuildError::ApplicationProgramValidation)?;
    let revisions = [
        *program.revision(),
        *successor.revision(),
        *successor_p2.revision(),
    ];
    let declaration =
        worth_query_host::facade::declaration::application_schema::fragment::declare(BankSchema)
            .map_err(|cause| BankIdentityRuntimeBuildError::SchemaDeclaration(Box::new(cause)))?;
    let index = std::sync::Arc::new(
        crate::application_definition::current_accounts::AccountIndexWork::default(),
    );
    Ok(WorthQueryApplicationHomeRequest::new(
        location,
        program,
        WorthQueryApplicationProgramRoster::new()
            .support(successor)
            .support(successor_p2),
        declaration,
        (index, (), ()),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID"))
            .expect("embedded application build identity"),
        installation::application_limits(product_world_resources::bank_product_world_resources()),
        move |_phase, graph, installed| {
            installation::bind_principals(graph, installed, principals)?;
            crate::graph_bootstrap::bind_program_deployment_scopes(graph, &revisions)?;
            installation::bind_world_truth(graph, Some(&world))
        },
    ))
}

/// A borrowed Bank command/authentication view; it neither clones nor owns Home.
pub struct BankHomeRuntime<'home> {
    runtime: &'home WorthQueryProgramApplicationRuntime<BankSchema, BankApplication>,
    binding: InstalledBankPrincipalBinding,
}
impl<'home> BankHomeRuntime<'home> {
    pub fn attach(
        runtime: &'home WorthQueryProgramApplicationRuntime<BankSchema, BankApplication>,
    ) -> Result<Self, BankIdentityRuntimeBuildError> {
        Ok(Self {
            runtime,
            binding: installation::resolve_installed_principal_binding(runtime.runtime())?,
        })
    }
    pub fn runtime(
        &self,
    ) -> &'home WorthQueryProgramApplicationRuntime<BankSchema, BankApplication> {
        self.runtime
    }
    pub fn request<'principal, 'scope>(
        &self,
        principal: &'principal BankAuthenticatedPrincipal,
        scope: &'scope WorthQueryRequestScope,
    ) -> WorthQueryApplicationRequest<'home, 'principal, 'scope, BankSchema> {
        self.runtime.request(principal.external(), scope)
    }
    pub fn admit_authentication_adapter<Adapter: WorthQueryAuthenticationAdapter>(
        &self,
        configuration: BankAuthenticationConfiguration,
        adapter: Adapter,
    ) -> Result<BankAuthenticationBoundary<Adapter>, BankAuthenticationBoundaryBuildError> {
        admit_authentication_adapter(self.runtime.runtime(), configuration, adapter)
    }
    pub async fn authenticate_with<Adapter: WorthQueryAuthenticationAdapter>(
        &self,
        authentication: &BankAuthenticationBoundary<Adapter>,
        credential: Adapter::Credential,
        scope: &WorthQueryRequestScope,
    ) -> Result<BankAuthenticatedPrincipal, BankPrincipalAdmissionError> {
        authenticate_with(
            self.runtime.runtime(),
            &self.binding,
            authentication,
            credential,
            scope,
        )
        .await
    }
    pub fn prepare_send_money_correction<'a>(
        &'a self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
        source: &worth_query_host::facade::primary_graph::WorthQueryCorrectionSourceCoordinates,
        key: &str,
        limits: worth_query_host::facade::application_entry::WorthQueryCorrectionHistoryLimits,
    ) -> Result<
        crate::BankSendMoneyCorrectionPreparation<'a>,
        worth_query_host::facade::application_entry::WorthQueryApplicationCorrectionDenial,
    > {
        crate::correction::prepare_send_money_correction(
            self.runtime,
            principal,
            scope,
            source,
            key,
            limits,
        )
    }
}

pub(super) fn admit_authentication_adapter<Adapter>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    configuration: BankAuthenticationConfiguration,
    adapter: Adapter,
) -> Result<BankAuthenticationBoundary<Adapter>, BankAuthenticationBoundaryBuildError>
where
    Adapter: WorthQueryAuthenticationAdapter,
{
    worth_query_host::facade::admission::authenticated_principal::admit_authentication_adapter(
            runtime.installed_schema(),
            WorthQueryAuthenticationAdapterAdmission::new(
                configuration.audience,
                configuration.method,
                worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
            ),
            adapter,
        )
        .map(BankAuthenticationBoundary::new)
        .map_err(BankAuthenticationBoundaryBuildError)
}

pub(super) async fn authenticate_with<Adapter>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    binding: &InstalledBankPrincipalBinding,

    authentication: &BankAuthenticationBoundary<Adapter>,
    credential: Adapter::Credential,
    scope: &WorthQueryRequestScope,
) -> Result<BankAuthenticatedPrincipal, BankPrincipalAdmissionError>
where
    Adapter: WorthQueryAuthenticationAdapter,
{
    let external = authentication
        .authenticate(credential, scope)
        .await
        .map_err(BankPrincipalAdmissionError::Authentication)?;
    runtime
        .with_application_advancement(scope, |phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .map_err(BankPrincipalAdmissionError::ProductSelection)?;
            let query = selected
                .resolve_authenticated_principal(
                    &phase,
                    binding,
                    &external,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .map_err(BankPrincipalAdmissionError::Resolution)?;
            let principal_id = *query.principal_identity();
            Ok(BankAuthenticatedPrincipal::new(
                principal_id,
                external,
                query,
            ))
        })
        .map_err(BankPrincipalAdmissionError::ExecutionRequest)?
}
