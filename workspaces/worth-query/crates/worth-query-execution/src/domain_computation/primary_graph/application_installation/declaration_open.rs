//! The declaration entry: contribution-owned schema meaning without a program.

use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::open_core::open_with_contributions;
use super::open_plan::{HomeStart, InitialState, OpenEntry, OpenPlan};
use super::open_refusal::OpenFailure;
use super::{
    WorthQueryApplicationLimits, WorthQueryApplicationOpenDenial, WorthQueryApplicationOpenRefusal,
};
use crate::domain_computation::primary_graph::{
    ApplicationHome, WorthQueryApplicationContributionTuple,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};
use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource;

/// One declaration open being described. `open` consumes it and the home.
pub struct WorthQueryDeclarationOpen<'open, Schema>
where
    Schema: ApplicationSchemaComposition,
    Schema::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration:
        <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryApplicationLimits,
    authorization_time_source: Option<Box<dyn WorthQueryRuntimeTimeSource>>,
    initial_state: InitialState<'open, Schema>,
}

/// Describes opening contribution-owned schema meaning without an application program.
pub fn declaration<'open, Schema>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryApplicationLimits,
) -> WorthQueryDeclarationOpen<'open, Schema>
where
    Schema: ApplicationSchemaComposition,
    Schema::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    WorthQueryDeclarationOpen {
        declaration,
        configuration,
        limits,
        authorization_time_source: None,
        initial_state: Box::new(|_, _| Ok(())),
    }
}

impl<'open, Schema> WorthQueryDeclarationOpen<'open, Schema>
where
    Schema: ApplicationSchemaComposition,
    Schema::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
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

    /// Opens the application on `home`. A refusal returns the home by phase.
    pub fn open(
        self,
        home: ApplicationHome,
    ) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryApplicationOpenRefusal>
    {
        let start = match home.closed_image() {
            Ok(image) => HomeStart::of(image, self.initial_state, None),
            Err(absent) => {
                return Err(
                    OpenFailure::from(WorthQueryApplicationOpenDenial::Home(absent)).refuse(home),
                )
            }
        };
        open_with_contributions::<Schema, Schema::Contributions>(
            self.declaration,
            self.configuration,
            self.limits,
            OpenPlan {
                entry: OpenEntry::Declaration,
                start,
                authorization_time_source: self.authorization_time_source,
            },
        )
        .map(|(runtime, _started)| runtime)
        .map_err(|failure| failure.refuse(home))
    }
}

/// Installs contribution-owned schema meaning without an application program.
///
/// Retained until every caller opens through [`declaration`].
pub fn in_memory<Schema>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryApplicationOpenDenial>
where
    Schema: ApplicationSchemaComposition,
    Schema::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    self::declaration(declaration, configuration, limits)
        .initial_state(initial_state)
        .open(ApplicationHome::memory())
        .map_err(|refusal| refusal.denial)
}
