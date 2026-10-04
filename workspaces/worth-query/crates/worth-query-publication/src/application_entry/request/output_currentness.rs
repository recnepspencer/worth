//! Current-output lineage admission at one caller-selected retained observation.
use super::super::WorthQueryApplicationRetainedRequest;
use std::num::NonZeroUsize;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

/// Why an output currentness check refused: the retained observation could not be
/// selected, it is on another branch, or an output no longer retains its source lineage at
/// that observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputCurrentnessDenial {
    Observation(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    ForeignBranch,
    Output(worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial),
}

/// Compatibility name for the earlier program-only currentness surface.
pub type WorthQueryProgramOutputCurrentnessDenial = WorthQueryOutputCurrentnessDenial;

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    /// Requires every output in a program settlement to retain its Query-owned
    /// source lineage at this request's one exact World observation.
    pub fn require_current_program_output<RootQuery>(
        &self,
        settlement: &crate::application_entry::WorthQueryApplicationProgramOutputSettlement<
            RootQuery,
        >,
        maximum_work: NonZeroUsize,
    ) -> Result<(), WorthQueryProgramOutputCurrentnessDenial> {
        self.require_current_output_settlements(settlement.retained_settlements(), maximum_work)
    }

    /// Requires a direct root or dependent demand's retained output lineage at
    /// this exact observation. Receipt-free checkpoint settlements use the same
    /// native authority check; stored fields alone never establish currentness.
    pub fn require_current_output_demand<Query>(
        &self,
        settlement: &crate::application_entry::WorthQueryApplicationOutputDemandSettlement<Query>,
        maximum_work: NonZeroUsize,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        self.require_current_output_settlements([settlement.retained()], maximum_work)
    }

    fn require_current_output_settlements<'settlement>(
        &self,
        settlements: impl IntoIterator<Item = &'settlement worth_query_execution::facade::primary_graph::WorthQueryOutputDemandSettlement>,
        maximum_work: NonZeroUsize,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        let selected = self
            .application
            .select_application_read_observation(&self.observation)
            .map_err(WorthQueryOutputCurrentnessDenial::Observation)?;
        if selected.product().product_branch() != self.branch {
            return Err(WorthQueryOutputCurrentnessDenial::ForeignBranch);
        }
        selected
            .require_current_output_settlements(settlements, maximum_work)
            .map_err(WorthQueryOutputCurrentnessDenial::Output)
    }
}
