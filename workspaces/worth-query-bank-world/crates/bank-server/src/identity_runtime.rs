use std::collections::BTreeSet;

#[path = "identity_runtime/installation.rs"]
mod installation;
mod product_world_resources;

use bank_domain::estate::BankEstateWorld;
use bank_domain::model::BankPrincipalId;
use bank_domain::proposals::BankSnapshot;
use bank_domain::schema::{BankPrincipalBinding, BankSchema, ExternalPrincipalMapping, Principal};
use worth_query_host::facade::admission::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAuthenticationAdapter,
    WorthQueryAuthenticationAdapterAdmission, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationMethod, WorthQueryRequestScope,
};
use worth_query_host::facade::declaration::application_schema::ApplicationOperationRef;
use worth_query_host::facade::domain::{
    WorthQueryInstalledAftermathContract, WorthQueryInstalledPrincipalBinding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationInvariantProjectionAuthority, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode, WorthQueryRuntimeTimeSource,
};

use crate::error::{
    BankAuthenticationBoundaryBuildError, BankIdentityRuntimeBuildError,
    BankPrincipalAdmissionError, BankWorldSeedDenial,
};
use crate::principal_seed::{BankPrincipalSeed, PreparedBankPrincipalSeed};
use crate::{
    BankApplicationQueryDenial, BankAuthenticatedPrincipal, BankAuthenticationBoundary,
    BankBusinessOwnerSeed, BankEmployeeAssignmentSeed, BankPreviewSession, BankWorldSeed,
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

pub struct BankIdentityRuntime {
    runtime: WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    binding: WorthQueryInstalledPrincipalBinding<
        BankSchema,
        BankPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        BankPrincipalId,
    >,
    invariant_projection: WorthQueryApplicationInvariantProjectionAuthority<BankSchema>,
}

impl BankIdentityRuntime {
    pub fn install(
        seeds: impl IntoIterator<Item = BankPrincipalSeed>,
    ) -> Result<Self, BankIdentityRuntimeBuildError> {
        let seeds = prepare_seeds(seeds)?;
        installation::install_prepared(
            seeds,
            None,
            installation::BankAuthorizationTimeInstallation::System,
        )
    }

    pub fn install_world(seed: BankWorldSeed) -> Result<Self, BankIdentityRuntimeBuildError> {
        let (principals, world) = prepare_world(seed)?;
        installation::install_prepared(
            principals,
            Some(world),
            installation::BankAuthorizationTimeInstallation::System,
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
        let query = self
            .runtime
            .resolve_authenticated_principal(
                &self.binding,
                external,
                scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(BankPrincipalAdmissionError::Resolution)?;
        let principal_id = *query.principal_identity();
        Ok(BankAuthenticatedPrincipal::new(principal_id, query))
    }

    pub fn validate(
        &self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
    ) -> Result<(), BankPrincipalAdmissionError> {
        self.runtime
            .validate_authenticated_principal(principal.query(), scope)
            .map_err(BankPrincipalAdmissionError::Resolution)
    }

    pub fn mapped_principal_count(&self) -> usize {
        self.runtime.publication().principal_binding_count()
    }

    pub fn open_preview(
        &self,
        request: &WorthQueryRequestScope,
    ) -> Result<BankPreviewSession, BankApplicationQueryDenial> {
        self.runtime
            .open_application_preview_session(request)
            .map(BankPreviewSession::from_query)
            .map_err(BankApplicationQueryDenial::from_preview_session)
    }

    pub(crate) const fn application_runtime(
        &self,
    ) -> &WorthQueryPrimaryGraphApplicationRuntime<BankSchema> {
        &self.runtime
    }

    /// Installed aftermath from the live bank schema — integration tests only.
    #[doc(hidden)]
    pub fn installed_operation_aftermath<Operation, Input>(
        &self,
        operation: ApplicationOperationRef<BankSchema, Operation, Input>,
    ) -> WorthQueryInstalledAftermathContract {
        self.application_runtime()
            .installed_schema()
            .installed_operation(operation)
            .expect("operation installed")
            .contracts()
            .aftermath()
            .expect("aftermath declared")
            .clone()
    }

    pub(crate) const fn invariant_projection(
        &self,
    ) -> &WorthQueryApplicationInvariantProjectionAuthority<BankSchema> {
        &self.invariant_projection
    }
}

struct BankGraphSeed {
    snapshot: BankSnapshot,
    owners: Vec<BankBusinessOwnerSeed>,
    employees: Vec<BankEmployeeAssignmentSeed>,
    estate: Option<BankEstateWorld>,
}

fn prepare_world(
    seed: BankWorldSeed,
) -> Result<(Vec<PreparedBankPrincipalSeed>, BankGraphSeed), BankIdentityRuntimeBuildError> {
    let (snapshot, principals, owners, employees, estate) = seed.into_parts();
    let principals = prepare_seeds(principals)?;
    validate_world_principals(&snapshot, &principals)?;
    Ok((
        principals,
        BankGraphSeed {
            snapshot,
            owners,
            employees,
            estate,
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
