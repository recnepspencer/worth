//! Admit the package, installed schema, and entry before contribution configuration.
use super::super::home_start::AdmittedEntry;
use super::super::open_plan::OpenEntry;
use super::super::{WorthQueryApplicationLimits, WorthQueryApplicationOpenDenial as Denial};
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryApplicationContractCatalog, WorthQueryApplicationContributionTuple,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

pub(super) struct AdmittedInstallation<Schema> {
    pub(super) runtime: WorthQueryExecutionRuntime,
    pub(super) authority: WorthQueryExecutionInstallationAuthority,
    pub(super) installed: WorthQueryInstalledApplicationSchema<Schema>,
    pub(super) entry: AdmittedEntry<Schema>,
    pub(super) contracts: WorthQueryApplicationContractCatalog<Schema>,
}

pub(super) fn admit<Schema, Contributions>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    limits: &WorthQueryApplicationLimits,
    entry: OpenEntry<'_, Schema>,
) -> Result<AdmittedInstallation<Schema>, Denial>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        Schema::MAJOR,
        Schema::MINOR,
    ));
    let contracts = Contributions::contracts().map_err(Denial::Contributions)?;
    let package = contracts
        .compose_package(package.application_schema(declaration.clone()))
        .validate()
        .map_err(Denial::Package)?;
    let admitted = WorthQueryInstallationAdmissionProfile::new(
        "primary-graph-in-memory",
        "application-contributions",
    )
    .admit(package)
    .map_err(Denial::Admission)?;
    let (runtime, authority) = WorthQueryExecutionRuntimeInstaller::new()
        .application_candidate_resources(limits.candidates)
        .application_query_resources(limits.queries)
        .output_demand_resources(limits.output_demands)
        .completed_evidence_resources(limits.completed_evidence)
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .map_err(Denial::Runtime)?
        .into_parts();
    let installed = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .map_err(Denial::Schema)?;
    let entry = match entry {
        OpenEntry::Declaration => {
            if installed
                .installed_mutation_binding_inventory()
                .any(|binding| binding.requires_workflow_authority())
            {
                return Err(Denial::WorkflowAuthorityRequiresProgram);
            }
            AdmittedEntry::Declaration
        }
        OpenEntry::Program(admit) => AdmittedEntry::Program(admit(&installed)?),
    };
    Ok(AdmittedInstallation {
        runtime,
        authority,
        installed,
        entry,
        contracts,
    })
}
