//! Installed meaning and closed program support precede every Native owner.
use super::super::application_contribution::{
    WorthQueryApplicationContributionTuple, WorthQueryConfiguredApplicationContributions,
};
use super::super::application_installation::program_admission::WorthQueryAdmittedProgramSupport;
use super::{
    WorthQueryApplicationLimits, WorthQueryApplicationPreparationDenial as Denial,
    WorthQueryProgramAdmissionStep,
};
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
    WorthQueryExecutionRuntimeInstaller,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstalledApplicationSchema, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

pub(super) struct PreparedInstalledMeaning<Schema: ApplicationSchemaComposition> {
    pub(super) runtime: WorthQueryExecutionRuntime,
    pub(super) authority: WorthQueryExecutionInstallationAuthority,
    pub(super) installed: WorthQueryInstalledApplicationSchema<Schema>,
    pub(super) admitted_program_support: Option<WorthQueryAdmittedProgramSupport<Schema>>,
    pub(super) configured: WorthQueryConfiguredApplicationContributions<Schema>,
}

pub(super) fn prepare<Schema, Contributions>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: &WorthQueryApplicationLimits,
    program_admission: Option<WorthQueryProgramAdmissionStep<'_, Schema>>,
    directory_support: Option<fn(worth_foundational::facade::DurableStateFamily, u8) -> Option<u8>>,
) -> Result<PreparedInstalledMeaning<Schema>, Denial>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        Schema::MAJOR,
        Schema::MINOR,
    ));
    let contracts =
        Contributions::contracts(declaration.erased()).map_err(Denial::Contributions)?;
    if let Some(support) = directory_support {
        contracts
            .admit_home_seed(declaration.erased(), support)
            .map_err(Denial::UnsupportedSeed)?;
    }
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
        .application_query_resources(limits.queries)
        .output_demand_resources(limits.output_demands)
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .map_err(Denial::Runtime)?
        .into_parts();
    let installed = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .map_err(Denial::Schema)?;
    if program_admission.is_none()
        && installed
            .installed_mutation_binding_inventory()
            .any(|binding| binding.requires_workflow_authority())
    {
        return Err(Denial::WorkflowAuthorityRequiresProgram);
    }
    let admitted_program_support = match program_admission {
        Some(admit) => Some(admit(&installed)?),
        None => None,
    };
    if let Some(support) = &admitted_program_support {
        contracts
            .validate_required_features(&support.features)
            .map_err(Denial::Contributions)?;
    }
    let configured = WorthQueryConfiguredApplicationContributions::<Schema>::configure::<
        Contributions,
    >(&installed, configuration, contracts)
    .map_err(Denial::Contributions)?;
    Ok(PreparedInstalledMeaning {
        runtime,
        authority,
        installed,
        admitted_program_support,
        configured,
    })
}
