//! Route only the actual Store origin into its closed Home recipe.
use super::{refusal::*, request::*, runtime::*, support::directory_requirement_refusal};
use worth_proof::{AuthorityMarker, CustodyReadmitted, TransitionSettled};
use worth_query_declaration::facade::{application_program::*, application_schema::*};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryApplicationContributionTuple, WorthQueryBootstrapAdvancementPhase,
    },
    application_home_integration::*,
    application_installation::*,
};
use worth_store_application::*;

impl<'authoring, Schema, Program> WorthQueryApplicationHomeRequest<'authoring, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    #[inline(never)]
    pub fn open_in_advancement(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryApplicationHome<Schema, Program>,
        WorthQueryApplicationHomeOpenRefused<Schema, Program>,
    > {
        match &self.location {
            WorthQueryApplicationHomeLocation::MemoryOnly => memory::open_memory(self, phase),
            WorthQueryApplicationHomeLocation::Directory(_) => {
                let opening = preparation::prepare_directory(self, phase)?;
                initialization::initialize_directory(phase, opening)
            }
        }
    }
}

mod custody;
pub(super) mod existing;
mod fresh;
pub(super) mod initialization;
mod memory;
pub(super) mod preparation;
pub(super) mod service;
