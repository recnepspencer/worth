//! Complete declaration and trusted service inputs for one Host opening.
use worth_query_declaration::facade::{
    application_program::{ApplicationProgramDefinition, ValidatedApplicationProgram},
    application_schema::{ApplicationSchemaComposition, ApplicationSchemaDeclaration},
};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryApplicationContributionTuple, WorthQueryBootstrapAdvancementPhase,
    },
    application_installation::{WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster},
    primary_graph::{WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

pub enum WorthQueryApplicationHomeLocation {
    MemoryOnly,
    Directory(std::path::PathBuf),
}

pub struct WorthQueryApplicationHomeRequest<
    'authoring,
    Schema,
    Program,
    Transition = super::transition::NoHomeProgramTransition,
> where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    pub(super) transition: Transition,
    pub(super) location: WorthQueryApplicationHomeLocation,
    pub(super) program: ValidatedApplicationProgram<Schema, Program>,
    pub(super) roster: WorthQueryApplicationProgramRoster<'authoring, Schema>,
    pub(super) declaration: ApplicationSchemaDeclaration<Schema>,
    pub(super) configuration:
        <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    pub(super) build: worth_foundational::facade::ApplicationBuildIdentity,
    pub(super) limits: WorthQueryApplicationLimits,
    pub(super) checkpoint_limits: worth_store_application::ApplicationHomeCheckpointLimits,
    pub(super) opening_control:
        Option<worth_store_application::ApplicationHomeOpeningControlForTest>,
    pub(super) authorization_time_source:
        Option<Box<dyn worth_query_execution::facade::primary_graph::WorthQueryRuntimeTimeSource>>,
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
        limits: WorthQueryApplicationLimits,
        initial_state: impl FnOnce(
                &WorthQueryBootstrapAdvancementPhase<'_>,
                &mut WorthQueryPrimaryGraphBootstrap<Schema>,
                &WorthQueryInstalledApplicationSchema<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
            + 'authoring,
    ) -> Self {
        Self {
            transition: super::transition::NoHomeProgramTransition,
            location,
            program,
            roster,
            declaration,
            configuration,
            build,
            limits,
            checkpoint_limits: Default::default(),
            opening_control: None,
            authorization_time_source: None,
            initial_state: Box::new(initial_state),
        }
    }
}
impl<'authoring, Schema, Program, Transition>
    WorthQueryApplicationHomeRequest<'authoring, Schema, Program, Transition>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    /// Count or payload bytes in any one Native/World journal make a checkpoint
    /// due. Both thresholds are finite and nonzero; the original request still
    /// controls all work and memory. Defaults are 4096 entries or 64 MiB.
    pub fn with_checkpoint_limits(
        mut self,
        limits: worth_store_application::ApplicationHomeCheckpointLimits,
    ) -> Self {
        self.checkpoint_limits = limits;
        self
    }
    /// Carries sealed Store certification placement into this actual opening.
    #[doc(hidden)]
    pub fn with_opening_control_for_test(
        mut self,
        control: worth_store_application::ApplicationHomeOpeningControlForTest,
    ) -> Self {
        self.opening_control = Some(control);
        self
    }

    /// Supplies the trusted clock used when admitting and checking capabilities.
    pub fn with_authorization_time_source(
        mut self,
        source: impl worth_query_execution::facade::primary_graph::WorthQueryRuntimeTimeSource,
    ) -> Self {
        self.authorization_time_source = Some(Box::new(source));
        self
    }
}
