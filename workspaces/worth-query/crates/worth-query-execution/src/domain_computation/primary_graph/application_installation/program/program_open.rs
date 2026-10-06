//! The program entry: one validated program, its roster, and the home it opens on.

use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::super::open_plan::{HomeStart, InitialState};
use super::super::open_refusal::OpenFailure;
use super::super::{
    WorthQueryApplicationLimits, WorthQueryApplicationOpenDenial, WorthQueryApplicationOpenRefusal,
    WorthQueryOpenAdoption,
};
use super::construction::open_program;
use super::{
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::{
    ApplicationHome, WorthQueryApplicationContributionTuple, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};
use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource;

/// One program open being described. `open` consumes it and the home.
pub struct WorthQueryProgramOpen<'open, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'open, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration:
        <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryApplicationLimits,
    authorization_time_source: Option<Box<dyn WorthQueryRuntimeTimeSource>>,
    initial_state: InitialState<'open, Schema>,
    adoption: Option<WorthQueryOpenAdoption<'open, Schema>>,
}

/// Describes opening one validated program and the application runtime it governs.
pub fn program<'open, Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryApplicationLimits,
) -> WorthQueryProgramOpen<'open, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    WorthQueryProgramOpen {
        program,
        roster: WorthQueryApplicationProgramRoster::new(),
        declaration,
        configuration,
        limits,
        authorization_time_source: None,
        initial_state: Box::new(|_, _| Ok(())),
        adoption: None,
    }
}

impl<'open, Schema, Program> WorthQueryProgramOpen<'open, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    /// Rosters the other programs this host supports.
    ///
    /// Every rostered program is admitted against the same installed schema, so
    /// closing the roster proves that no installed rule is left without a
    /// declaring owner even when no single program declares them all. A home
    /// resumes under any rostered revision its image recorded.
    pub fn roster(mut self, roster: WorthQueryApplicationProgramRoster<'open, Schema>) -> Self {
        self.roster = roster;
        self
    }

    /// Fixes one host-owned trusted-time source before publication.
    pub fn authorization_time_source(mut self, source: impl WorthQueryRuntimeTimeSource) -> Self {
        self.authorization_time_source = Some(Box::new(source));
        self
    }

    /// Declares the state an empty home starts with. A resumed home skips it.
    pub fn initial_state(
        mut self,
        seed: impl FnOnce(
                &mut WorthQueryPrimaryGraphBootstrap<Schema>,
                &WorthQueryInstalledApplicationSchema<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
            + 'open,
    ) -> Self {
        self.initial_state = Box::new(seed);
        self
    }

    /// Declares the adoption to run when the home's image names its predecessor.
    ///
    /// This first adoption surface supports new records and links between
    /// them; it refuses retained outputs, workflow records and relation-scoped
    /// rules until their migration owners supply complete support.
    pub fn adopt_on_open(mut self, adoption: WorthQueryOpenAdoption<'open, Schema>) -> Self {
        self.adoption = Some(adoption);
        self
    }

    /// Opens the application on `home`. A refusal returns the home by phase.
    pub fn open(
        self,
        home: ApplicationHome,
    ) -> Result<
        WorthQueryProgramApplicationRuntime<Schema, Program>,
        WorthQueryApplicationOpenRefusal,
    > {
        let start = match home.closed_image() {
            Ok(image) => HomeStart::of(image, self.initial_state, self.adoption),
            Err(absent) => {
                return Err(
                    OpenFailure::from(WorthQueryApplicationOpenDenial::Home(absent)).refuse(home),
                )
            }
        };
        open_program(
            self.program,
            self.roster,
            self.declaration,
            self.configuration,
            self.limits,
            start,
            self.authorization_time_source,
        )
        .map_err(|failure| failure.refuse(home))
    }
}
