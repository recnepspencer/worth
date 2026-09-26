//! Authoring the set of programs one host rosters beside its initial program.
//!
//! Rostered programs are authored in different Rust types, so the host cannot
//! hold them in one homogeneous collection. It holds each one behind a contract
//! written while that program's type is still known, which is how a roster of
//! heterogeneous programs stays typed instead of collapsing into strings or
//! untyped identifiers.

use std::any::TypeId;
use std::collections::BTreeSet;

use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_installation::facade::{
    install_rostered_application_program, ApplicationSchema, WorthQueryInstalledApplicationSchema,
    WorthQueryProgramSupportAdmission, WorthQueryProgramSupportDenial,
    WorthQueryProgramSupportRoster,
};

use super::supported_program::WorthQuerySupportedProgramRecord;
use super::WorthQueryApplicationProgramRoots;

/// One program a host rosters, contracted while its authoring type is known.
trait WorthQueryRosteredProgram<Schema> {
    /// Admits this program into support without consuming it, so the same
    /// program can still be installed once the roster closes.
    fn admit<'installation>(
        &self,
        admission: WorthQueryProgramSupportAdmission<'installation, Schema>,
    ) -> Result<
        WorthQueryProgramSupportAdmission<'installation, Schema>,
        WorthQueryProgramSupportDenial,
    >;

    /// Installs this program against the closed roster and records what its
    /// installation authorizes.
    fn install(
        self: Box<Self>,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        roster: &WorthQueryProgramSupportRoster<Schema>,
    ) -> Result<WorthQuerySupportedProgramRecord, WorthQueryProgramSupportDenial>;
}

struct WorthQueryRosteredProgramAuthoring<Schema, Program> {
    program: ValidatedApplicationProgram<Schema, Program>,
}

impl<Schema, Program> WorthQueryRosteredProgram<Schema>
    for WorthQueryRosteredProgramAuthoring<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    fn admit<'installation>(
        &self,
        admission: WorthQueryProgramSupportAdmission<'installation, Schema>,
    ) -> Result<
        WorthQueryProgramSupportAdmission<'installation, Schema>,
        WorthQueryProgramSupportDenial,
    > {
        admission.support(&self.program)
    }

    fn install(
        self: Box<Self>,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        roster: &WorthQueryProgramSupportRoster<Schema>,
    ) -> Result<WorthQuerySupportedProgramRecord, WorthQueryProgramSupportDenial> {
        let mut output_sources = BTreeSet::new();
        Program::Outputs::append_required_bindings(&mut output_sources);
        output_sources.extend(self.program.actions().iter().filter_map(|action| {
            action
                .required_output_source()
                .then(|| action.mutation_binding_type())
                .flatten()
        }));
        let installed =
            install_rostered_application_program(self.program, installed_schema, roster)?;
        Ok(WorthQuerySupportedProgramRecord::installed(
            installed,
            &output_sources.into_iter().collect::<Vec<TypeId>>(),
            &Program::Outputs::root_graph_types(),
        ))
    }
}

/// The additional programs a host rosters beside its initial program.
///
/// Each program is added while its authoring type is still known, so closing
/// the roster yields typed installed products rather than erased records the
/// host would have to take on trust.
pub struct WorthQueryApplicationProgramRoster<'authoring, Schema> {
    rostered: Vec<Box<dyn WorthQueryRosteredProgram<Schema> + 'authoring>>,
}

impl<Schema> Default for WorthQueryApplicationProgramRoster<'_, Schema> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'authoring, Schema> WorthQueryApplicationProgramRoster<'authoring, Schema> {
    pub const fn new() -> Self {
        Self {
            rostered: Vec::new(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.rostered.is_empty()
    }

    /// Rosters one further validated program beside the initial one.
    pub fn support<Program>(mut self, program: ValidatedApplicationProgram<Schema, Program>) -> Self
    where
        Schema: ApplicationSchema + 'authoring,
        Program: ApplicationProgramDefinition<Schema> + 'static,
        Program::Outputs:
            ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    {
        self.rostered
            .push(Box::new(WorthQueryRosteredProgramAuthoring { program }));
        self
    }

    pub(in crate::domain_computation::primary_graph) fn admit_all<'installation>(
        &self,
        mut admission: WorthQueryProgramSupportAdmission<'installation, Schema>,
    ) -> Result<
        WorthQueryProgramSupportAdmission<'installation, Schema>,
        WorthQueryProgramSupportDenial,
    > {
        for rostered in &self.rostered {
            admission = rostered.admit(admission)?;
        }
        Ok(admission)
    }

    pub(in crate::domain_computation::primary_graph) fn install_all(
        self,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        roster: &WorthQueryProgramSupportRoster<Schema>,
    ) -> Result<Vec<WorthQuerySupportedProgramRecord>, WorthQueryProgramSupportDenial> {
        self.rostered
            .into_iter()
            .map(|rostered| rostered.install(installed_schema, roster))
            .collect()
    }
}
