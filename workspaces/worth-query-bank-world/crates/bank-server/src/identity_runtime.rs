mod advancement;
mod workflow_binding;
use std::collections::BTreeSet;
pub use workflow_binding::BankIdentityRuntimeWorkflowBinding;

#[path = "identity_runtime/certification_controls.rs"]
mod certification_controls;
#[path = "identity_runtime/installation.rs"]
mod installation;
pub(crate) mod product_world_resources;

#[cfg(test)]
pub(crate) fn bank_application_limits(
) -> worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits {
    installation::bank_application_limits()
}

use bank_domain::estate::BankEstateWorld;
use bank_domain::model::BankPrincipalId;
use bank_domain::proposals::BankSnapshot;
use bank_domain::schema::{
    ApprovedBusinessPaymentWorkflow, BankPrincipalBinding, BankPrincipalIdBinding, BankSchema,
    ExternalPrincipalMapping, Principal,
};
use worth_query_host::facade::admission::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAuthenticationAdapter,
    WorthQueryAuthenticationAdapterAdmission, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationMethod, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequest, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryWorkflowApplicationRuntime,
};
use worth_query_host::facade::declaration::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationStructuredValueBinding,
};
use worth_query_host::facade::domain::{
    WorthQueryInstalledAftermathContract, WorthQueryInstalledPrincipalBinding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationInvariantProjectionAuthority, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode, WorthQueryRuntimeTimeSource,
};

use crate::application_definition::BankApplication;
use crate::approval_authentication::BankApprovalAuthenticationOwner;

use crate::error::{
    BankAuthenticationBoundaryBuildError, BankIdentityRuntimeBuildError,
    BankPrincipalAdmissionError, BankWorldSeedDenial,
};
use crate::principal_seed::{BankPrincipalSeed, PreparedBankPrincipalSeed};
use crate::{
    BankApplicationQueryDenial, BankApprovalAuthenticationConfiguration,
    BankAuthenticatedPrincipal, BankAuthenticationBoundary, BankBusinessOwnerSeed,
    BankEmployeeAssignmentSeed, BankPreviewSession, BankWorldSeed,
};

pub struct BankAuthenticationConfiguration {
    audience: WorthQueryAuthenticationAudience,
    method: WorthQueryAuthenticationMethod,
}

impl BankAuthenticationConfiguration {
    pub const fn new(
        audience: WorthQueryAuthenticationAudience,
        method: WorthQueryAuthenticationMethod,
    ) -> Self {
        Self { audience, method }
    }
}

type InstalledBankPrincipalBinding = WorthQueryInstalledPrincipalBinding<
    BankSchema,
    BankPrincipalBinding,
    ExternalPrincipalMapping,
    Principal,
    BankPrincipalId,
    BankPrincipalIdBinding,
>;

pub struct BankIdentityRuntime {
    runtime: WorthQueryWorkflowApplicationRuntime<
        BankSchema,
        ApprovedBusinessPaymentWorkflow,
        BankApplication,
    >,
    binding: InstalledBankPrincipalBinding,
    invariant_projection:
        std::sync::Arc<WorthQueryApplicationInvariantProjectionAuthority<BankSchema>>,
    approval_authentication: BankApprovalAuthenticationOwner,
}

impl BankIdentityRuntime {
    pub fn approved_business_payment<'runtime, 'principal, 'scope>(
        &'runtime self,
        principal: &'principal BankAuthenticatedPrincipal,
        scope: &'scope WorthQueryRequestScope,
    ) -> crate::BankApprovedPaymentWorkflow<'runtime, 'principal, 'scope> {
        crate::BankApprovedPaymentWorkflow::new(self, principal, scope)
    }

    pub(crate) const fn approved_payment_workflow_runtime(
        &self,
    ) -> &WorthQueryWorkflowApplicationRuntime<
        BankSchema,
        ApprovedBusinessPaymentWorkflow,
        BankApplication,
    > {
        &self.runtime
    }

    /// The installed Bank vocabulary used to readmit portable payment workflow
    /// definitions before they can be bound or published.
    pub const fn approved_payment_workflow_spec(
        &self,
    ) -> &worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
        BankSchema,
        ApprovedBusinessPaymentWorkflow,
    > {
        self.runtime.workflow_spec()
    }

    pub(crate) const fn approval_authentication(&self) -> &BankApprovalAuthenticationOwner {
        &self.approval_authentication
    }

    pub(crate) const fn application_program(
        &self,
    ) -> &WorthQueryProgramApplicationRuntime<BankSchema, BankApplication> {
        self.runtime.program_runtime()
    }

    pub fn request<'application, 'principal, 'scope>(
        &'application self,
        principal: &'principal BankAuthenticatedPrincipal,
        scope: &'scope WorthQueryRequestScope,
    ) -> WorthQueryApplicationRequest<'application, 'principal, 'scope, BankSchema> {
        self.runtime.request(principal.external(), scope)
    }

    pub fn install(
        seeds: impl IntoIterator<Item = BankPrincipalSeed>,
    ) -> Result<Self, BankIdentityRuntimeBuildError> {
        let seeds = prepare_seeds(seeds)?;
        installation::install_prepared(
            seeds,
            None,
            installation::BankAuthorizationTimeInstallation::System,
            BankApprovalAuthenticationConfiguration::denying(),
        )
    }

    pub fn install_world(seed: BankWorldSeed) -> Result<Self, BankIdentityRuntimeBuildError> {
        let (principals, world) = prepare_world(seed)?;
        installation::install_prepared(
            principals,
            Some(world),
            installation::BankAuthorizationTimeInstallation::System,
            BankApprovalAuthenticationConfiguration::denying(),
        )
    }

    /// Installs the host's trusted approval factor for the payment workflow.
    pub fn install_world_with_approval_authentication(
        seed: BankWorldSeed,
        approval_authentication: BankApprovalAuthenticationConfiguration,
    ) -> Result<Self, BankIdentityRuntimeBuildError> {
        let (principals, world) = prepare_world(seed)?;
        installation::install_prepared(
            principals,
            Some(world),
            installation::BankAuthorizationTimeInstallation::System,
            approval_authentication,
        )
    }

    /// Installs a world with one runtime-lifetime authorization-time source.
    ///
    /// The source is an external mechanism rather than Bank or Query
    /// authority. Operation callers cannot replace it or provide samples.
    pub fn install_world_with_authorization_time_source(
        seed: BankWorldSeed,
        source: impl WorthQueryRuntimeTimeSource,
    ) -> Result<Self, BankIdentityRuntimeBuildError> {
        let (principals, world) = prepare_world(seed)?;
        installation::install_prepared(
            principals,
            Some(world),
            installation::BankAuthorizationTimeInstallation::Installed(Box::new(source)),
            BankApprovalAuthenticationConfiguration::denying(),
        )
    }

    pub fn admit_authentication_adapter<Adapter>(
        &self,
        configuration: BankAuthenticationConfiguration,
        adapter: Adapter,
    ) -> Result<BankAuthenticationBoundary<Adapter>, BankAuthenticationBoundaryBuildError>
    where
        Adapter: WorthQueryAuthenticationAdapter,
    {
        admit_authentication_adapter(
            self.runtime.installed_schema(),
            WorthQueryAuthenticationAdapterAdmission::new(
                configuration.audience,
                configuration.method,
            ),
            adapter,
        )
        .map(BankAuthenticationBoundary::new)
        .map_err(BankAuthenticationBoundaryBuildError)
    }

    pub async fn authenticate_with<Adapter>(
        &self,

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
        self.application_runtime()
            .with_application_advancement(scope, |phase| {
                let selected = self
                    .select_current_product(&phase)
                    .map_err(BankPrincipalAdmissionError::ProductSelection)?;
                let query = selected
                    .resolve_authenticated_principal(
                        &phase,
                        &self.binding,
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

    pub fn validate(
        &self,

        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
    ) -> Result<(), BankPrincipalAdmissionError> {
        self.application_runtime()
            .with_application_advancement(scope, |phase| {
                self.select_current_product(&phase)
                    .map_err(BankPrincipalAdmissionError::ProductSelection)?
                    .validate_authenticated_principal(&phase, principal.query(), scope)
                    .map_err(BankPrincipalAdmissionError::Resolution)
            })
            .map_err(BankPrincipalAdmissionError::ExecutionRequest)?
    }

    pub fn mapped_principal_count(&self) -> usize {
        self.runtime.publication().principal_binding_count()
    }

    pub fn open_preview(
        &self,
        request: &WorthQueryRequestScope,
    ) -> Result<BankPreviewSession, BankApplicationQueryDenial> {
        self.application_runtime()
            .with_application_advancement(request, |phase| {
                BankPreviewSession::open(&phase, &self.runtime, request)
            })
            .map_err(BankApplicationQueryDenial::ExecutionRequest)?
    }

    pub(crate) fn application_runtime(
        &self,
    ) -> &WorthQueryPrimaryGraphApplicationRuntime<BankSchema> {
        self.runtime.program_runtime().runtime()
    }

    /// Installed aftermath from the live bank schema — integration tests only.
    #[doc(hidden)]
    pub fn installed_operation_aftermath<Operation, Input>(
        &self,
        operation: ApplicationOperationRef<BankSchema, Operation, Input>,
    ) -> WorthQueryInstalledAftermathContract
    where
        Operation: ApplicationOperationMarkerIdentity<BankSchema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        self.application_runtime()
            .installed_schema()
            .installed_operation(operation)
            .expect("operation installed")
            .contracts()
            .aftermath()
            .expect("aftermath declared")
            .clone()
    }

    pub(crate) fn invariant_projection(
        &self,
    ) -> &WorthQueryApplicationInvariantProjectionAuthority<BankSchema> {
        &self.invariant_projection
    }

    pub(crate) fn retain_invariant_projection(
        &self,
    ) -> std::sync::Arc<WorthQueryApplicationInvariantProjectionAuthority<BankSchema>> {
        std::sync::Arc::clone(&self.invariant_projection)
    }
}

struct BankGraphSeed {
    snapshot: BankSnapshot,
    owners: Vec<BankBusinessOwnerSeed>,
    employees: Vec<BankEmployeeAssignmentSeed>,
    estate: Option<BankEstateWorld>,
    deployment_grants: Vec<crate::BankProgramDeploymentGrantSeed>,
}

fn prepare_world(
    seed: BankWorldSeed,
) -> Result<(Vec<PreparedBankPrincipalSeed>, BankGraphSeed), BankIdentityRuntimeBuildError> {
    let (snapshot, principals, owners, employees, estate, deployment_grants) = seed.into_parts();
    let principals = prepare_seeds(principals)?;
    validate_world_principals(&snapshot, &principals)?;
    Ok((
        principals,
        BankGraphSeed {
            snapshot,
            owners,
            employees,
            estate,
            deployment_grants,
        },
    ))
}

fn prepare_seeds(
    seeds: impl IntoIterator<Item = BankPrincipalSeed>,
) -> Result<Vec<PreparedBankPrincipalSeed>, BankIdentityRuntimeBuildError> {
    seeds
        .into_iter()
        .map(|seed| {
            seed.prepare()
                .map_err(BankIdentityRuntimeBuildError::PrincipalKey)
        })
        .collect()
}

fn validate_world_principals(
    snapshot: &BankSnapshot,
    principals: &[PreparedBankPrincipalSeed],
) -> Result<(), BankIdentityRuntimeBuildError> {
    let expected = snapshot.principals().collect::<BTreeSet<_>>();
    let supplied = principals
        .iter()
        .map(|principal| principal.principal_id)
        .collect::<BTreeSet<_>>();
    if expected == supplied && supplied.len() == principals.len() {
        Ok(())
    } else {
        Err(BankIdentityRuntimeBuildError::WorldSeed(
            BankWorldSeedDenial::PrincipalSetMismatch,
        ))
    }
}
