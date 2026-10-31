//! Complete declaration and trusted service inputs for one Host opening.
use worth_query_declaration::facade::{
    application_program::{ApplicationProgramDefinition, ValidatedApplicationProgram},
    application_schema::{ApplicationSchemaComposition, ApplicationSchemaDeclaration},
};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryApplicationContributionTuple, WorthQueryBootstrapAdvancementPhase,
    },
    application_installation::{
        WorthQueryApplicationProgramRoster, WorthQueryInMemoryApplicationLimits,
    },
    primary_graph::{WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

pub enum WorthQueryApplicationHomeLocation {
    MemoryOnly,
    Directory(std::path::PathBuf),
}

pub struct WorthQueryApplicationHomeRequest<'authoring, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    pub(super) location: WorthQueryApplicationHomeLocation,
    pub(super) program: ValidatedApplicationProgram<Schema, Program>,
    pub(super) roster: WorthQueryApplicationProgramRoster<'authoring, Schema>,
    pub(super) declaration: ApplicationSchemaDeclaration<Schema>,
    pub(super) configuration:
        <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    pub(super) build: worth_foundational::facade::ApplicationBuildIdentity,
    pub(super) limits: WorthQueryInMemoryApplicationLimits,
    pub(super) initial_state: Box<
        dyn FnOnce(
                &WorthQueryBootstrapAdvancementPhase<'_>,
                &mut WorthQueryPrimaryGraphBootstrap<Schema>,
                &WorthQueryInstalledApplicationSchema<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
            + 'authoring,
    >,
}
impl<'authoring, Schema, Program> WorthQueryApplicationHomeRequest<'authoring, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    pub fn new(
        location: WorthQueryApplicationHomeLocation,
        program: ValidatedApplicationProgram<Schema, Program>,
        roster: WorthQueryApplicationProgramRoster<'authoring, Schema>,
        declaration: ApplicationSchemaDeclaration<Schema>,
        configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
        build: worth_foundational::facade::ApplicationBuildIdentity,
        limits: WorthQueryInMemoryApplicationLimits,
        initial_state: impl FnOnce(
                &WorthQueryBootstrapAdvancementPhase<'_>,
                &mut WorthQueryPrimaryGraphBootstrap<Schema>,
                &WorthQueryInstalledApplicationSchema<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
            + 'authoring,
    ) -> Self {
        Self {
            location,
            program,
            roster,
            declaration,
            configuration,
            build,
            limits,
            initial_state: Box::new(initial_state),
        }
    }
}
