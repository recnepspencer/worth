//! Closed Host integration stages; Store keeps physical and Native ownership.
use super::super::{
    application_contribution::PreparedApplicationRuntimeContributions,
    bootstrap::native_meaning::InstalledNativeSchemaMeaning,
    program_occurrence::WorthQueryProgramActivationCell,
};
use super::*;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationProgram;
use worth_relational::facade::durability::{RecoveredRelationalRuntimeAuthority, RecoveryOutcome};
use worth_relational::facade::runtime::{
    RelationalInitialSchemaInstallationReceipt, RelationalRuntime, RelationalRuntimeDefinition,
};
mod binding;
mod world;
pub use world::{QueryHomeWorldAttachmentRefused, ReadmittedHomeQueryService};
mod native;
mod source;
pub use native::{QueryHomeNativeInstallationRefused, QueryHomeNativeReadmissionRefused};
pub use source::{
    PreparedHomeQueryRuntime, QueryHomeInitialStateDenial, QueryHomeSourceAttachmentRefused,
};

pub struct PreparedHomeApplication<Schema: ApplicationSchemaComposition, Program> {
    state: HomeApplicationState<Schema, Program>,
    invariants: super::super::WorthQueryApplicationInvariantFactories<Schema>,
}

pub struct PreparedHomeNativeMeaning<Schema: ApplicationSchemaComposition, Program> {
    state: HomeApplicationState<Schema, Program>,
    schema: InstalledNativeSchemaMeaning,
}

pub struct ReadmittedHomeNativeMeaning<Schema: ApplicationSchemaComposition, Program> {
    state: HomeApplicationState<Schema, Program>,
    schema: InstalledNativeSchemaMeaning,
    restore_work: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
}

struct HomeApplicationState<Schema: ApplicationSchemaComposition, Program> {
    runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
    installed: WorthQueryInstalledApplicationSchema<Schema>,
    program: WorthQueryInstalledApplicationProgram<Schema, Program>,
    supported: Vec<WorthQuerySupportedProgramRecord>,
    support: program_admission::WorthQueryAdmittedProgramSupport<Schema>,
    pending: PreparedApplicationRuntimeContributions<Schema>,
    activation: WorthQueryProgramActivationCell,
    limits: WorthQueryInMemoryApplicationLimits,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
}

/// Installs actual schema/program meaning before Store construction starts.
/// This integration door is closed to the sole Host Home composition module.
pub fn prepare_home_program<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
) -> Result<PreparedHomeApplication<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let mut admitted = None;
    let prepared_meaning::PreparedInstalledMeaning {
        runtime,
        authority,
        installed,
        admitted_program_support,
        configured,
    } = prepared_meaning::prepare::<Schema, Program::Contributions>(
        declaration,
        configuration,
        &limits,
        Some(Box::new(|installed_schema| {
            let (support, installed, supported) =
                program::assembly::admit_program_roster(program, roster, installed_schema)?;
            admitted = Some((installed, supported));
            Ok(support)
        })),
    )?;
    let (invariants, pending, activation) = contribution_preparation::prepare(
        configured,
        &runtime,
        &installed,
        &admitted_program_support,
    )?;
    let support = admitted_program_support
        .ok_or(WorthQueryInMemoryApplicationDenial::ProgramAdmissionIncomplete)?;
    let (program, supported) =
        admitted.ok_or(WorthQueryInMemoryApplicationDenial::ProgramAdmissionIncomplete)?;
    Ok(PreparedHomeApplication {
        state: HomeApplicationState {
            runtime,
            authority,
            installed,
            program,
            supported,
            support,
            pending,
            activation,
            limits,
            build_identity,
        },
        invariants,
    })
}

impl<Schema: ApplicationSchemaComposition, Program> PreparedHomeApplication<Schema, Program> {
    pub fn home_relational_definition(&self) -> RelationalRuntimeDefinition {
        let limits = &self.state.limits;
        let mut builder = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
            .profile(limits.profile.relational_profile());
        if let Some(publication) = limits
            .profile
            .publication_override(limits.maximum_publication_records)
        {
            builder = builder.publication(publication);
        }
        builder.resolve()
    }
}

impl<Schema: ApplicationSchemaComposition, Program> PreparedHomeNativeMeaning<Schema, Program> {
    pub fn home_initial_schema_receipt(&self) -> &RelationalInitialSchemaInstallationReceipt {
        &self.schema.receipt
    }
}
