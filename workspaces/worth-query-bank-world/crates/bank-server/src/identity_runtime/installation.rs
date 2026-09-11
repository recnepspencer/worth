use bank_domain::{
    model::BankPrincipalId,
    schema::{
        BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, ExternalPrincipalMapping,
        Principal,
    },
};
use worth_query_host::facade::{
    domain::{
        WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
        WorthQueryInstalledApplicationSchema, WorthQueryInstalledPrincipalBinding,
    },
    primary_graph::{
        SignalConditionalEvaluationBudget, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryPrimaryGraphBootstrap, WorthQueryRuntimeTimeSource,
    },
    runtime::{
        WorthQueryApplicationQueryResourceProfile, WorthQueryExecutionInstallationAuthority,
        WorthQueryExecutionRuntime, WorthQueryExecutionRuntimeInstaller,
    },
};

use super::{BankGraphSeed, BankIdentityRuntime};
use crate::{
    domain_package::bank_domain_package, error::BankIdentityRuntimeBuildError,
    graph_bootstrap::bind_bank_world_with_estate, principal_seed::PreparedBankPrincipalSeed,
};

type InstalledBankPrincipalBinding = WorthQueryInstalledPrincipalBinding<
    BankSchema,
    BankPrincipalBinding,
    ExternalPrincipalMapping,
    Principal,
    BankPrincipalId,
    BankPrincipalIdBinding,
>;

pub(super) enum BankAuthorizationTimeInstallation {
    System,
    Installed(Box<dyn WorthQueryRuntimeTimeSource>),
}

struct PreparedBankGraph {
    graph: WorthQueryPrimaryGraphBootstrap<BankSchema>,
    runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
    installed_schema: WorthQueryInstalledApplicationSchema<BankSchema>,
}

impl PreparedBankGraph {
    fn publish_application_runtime(
        self,
        authorization_time: BankAuthorizationTimeInstallation,
    ) -> Result<WorthQueryPrimaryGraphApplicationRuntime<BankSchema>, BankIdentityRuntimeBuildError>
    {
        match authorization_time {
            BankAuthorizationTimeInstallation::System => self.graph.publish_application_runtime(
                self.runtime,
                self.authority,
                self.installed_schema,
                SignalConditionalEvaluationBudget::development(),
            ),
            BankAuthorizationTimeInstallation::Installed(source) => self
                .graph
                .publish_application_runtime_with_authorization_time_source(
                    self.runtime,
                    self.authority,
                    self.installed_schema,
                    SignalConditionalEvaluationBudget::development(),
                    source,
                ),
        }
        .map_err(BankIdentityRuntimeBuildError::PrimaryGraph)
    }
}

pub(super) fn install_prepared(
    seeds: Vec<PreparedBankPrincipalSeed>,
    world: Option<BankGraphSeed>,
    authorization_time: BankAuthorizationTimeInstallation,
) -> Result<BankIdentityRuntime, BankIdentityRuntimeBuildError> {
    let execution_runtime = install_bank_execution_runtime()?;
    let mut prepared_graph = prepare_seeded_primary_graph(execution_runtime, seeds)?;
    bind_world_truth(&mut prepared_graph.graph, world.as_ref())?;
    let invariant_projection = prepared_graph.graph.retain_invariant_projection_authority();
    let runtime = prepared_graph.publish_application_runtime(authorization_time)?;
    let binding = resolve_installed_principal_binding(&runtime)?;
    Ok(BankIdentityRuntime {
        runtime,
        binding,
        invariant_projection,
    })
}

fn install_bank_execution_runtime() -> Result<
    (
        WorthQueryExecutionRuntime,
        WorthQueryExecutionInstallationAuthority,
        WorthQueryInstalledApplicationSchema<BankSchema>,
    ),
    BankIdentityRuntimeBuildError,
> {
    let package =
        bank_domain_package().map_err(BankIdentityRuntimeBuildError::SchemaDeclaration)?;
    let validated = package
        .validate()
        .map_err(BankIdentityRuntimeBuildError::PackageValidation)?;
    let admitted = WorthQueryInstallationAdmissionProfile::new(
        "worth-query-primary-graph-host-v1",
        "bank-primary-graph-v1",
    )
    .admit(validated)
    .map_err(BankIdentityRuntimeBuildError::PackageAdmission)?;
    let application_query_resources =
        WorthQueryApplicationQueryResourceProfile::bounded(32_768, 32_768, 32_768, 64)
            .expect("bank application-query resource profile is statically non-zero");
    let installation = WorthQueryExecutionRuntimeInstaller::new()
        .application_query_resources(application_query_resources)
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .map_err(BankIdentityRuntimeBuildError::RuntimeInstallation)?;
    let (runtime, authority) = installation.into_parts();
    let installed_schema = runtime
        .installed_packages()
        .bind_application_schema(
            BankSchema::declaration().map_err(BankIdentityRuntimeBuildError::SchemaDeclaration)?,
        )
        .map_err(BankIdentityRuntimeBuildError::InstalledSchema)?;
    Ok((runtime, authority, installed_schema))
}

fn prepare_seeded_primary_graph(
    execution_runtime: (
        WorthQueryExecutionRuntime,
        WorthQueryExecutionInstallationAuthority,
        WorthQueryInstalledApplicationSchema<BankSchema>,
    ),
    seeds: Vec<PreparedBankPrincipalSeed>,
) -> Result<PreparedBankGraph, BankIdentityRuntimeBuildError> {
    let (runtime, authority, installed_schema) = execution_runtime;
    let mut graph = authority
        .prepare_primary_graph(
            &runtime,
            &installed_schema,
            super::product_world_resources::bank_product_world_resources(),
        )
        .map_err(BankIdentityRuntimeBuildError::PrimaryGraph)?;
    let binding = installed_schema
        .principal_binding(BankPrincipalBinding::reference())
        .map_err(BankIdentityRuntimeBuildError::InstalledBinding)?;
    for seed in seeds {
        graph
            .bind_principal(
                &binding,
                seed.key,
                seed.principal_id,
                seed.external_identity,
                seed.status,
            )
            .map_err(BankIdentityRuntimeBuildError::PrimaryGraph)?;
    }
    Ok(PreparedBankGraph {
        graph,
        runtime,
        authority,
        installed_schema,
    })
}

fn bind_world_truth(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    world: Option<&BankGraphSeed>,
) -> Result<(), BankIdentityRuntimeBuildError> {
    let Some(world) = world else {
        return Ok(());
    };
    bind_bank_world_with_estate(
        graph,
        &world.snapshot,
        &world.owners,
        &world.employees,
        world.estate.as_ref(),
    )
    .map_err(BankIdentityRuntimeBuildError::PrimaryGraph)
}

fn resolve_installed_principal_binding(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
) -> Result<InstalledBankPrincipalBinding, BankIdentityRuntimeBuildError> {
    runtime
        .installed_schema()
        .principal_binding(BankPrincipalBinding::reference())
        .map_err(BankIdentityRuntimeBuildError::InstalledBinding)
}
