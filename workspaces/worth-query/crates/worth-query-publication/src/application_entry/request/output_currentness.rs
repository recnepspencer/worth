//! Current-output lineage admission at one caller-selected retained observation.
use super::super::WorthQueryApplicationRetainedRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

/// Why an output currentness check refused: the retained observation could not be
/// selected, it is on another branch, or an output no longer retains its source lineage at
/// that observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputCurrentnessDenial {
    /// The call was refused before its retained-output validation read.
    ExecutionRequest(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    Observation(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    ForeignBranch,
    /// Original discovery validation, materialization or storage refused.
    Source(worth_query_execution::facade::primary_graph::WorthQuerySourceExpectationDenial),
    Output(worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial),
}

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    /// Requires every retained output and any original dependent discovery
    /// consumption at this request's one exact World observation. A fixed
    /// root with no discovery retains the native output-only meaning.
    pub fn require_current_program_output<RootQuery>(
        &self,
        settlement: &crate::application_entry::WorthQueryApplicationProgramOutputSettlement<
            RootQuery,
        >,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        self.application
            .with_application_advancement(self.scope, |phase| {
                self.require_current_program_output_in_advancement(&phase, settlement)
            })
            .map_err(WorthQueryOutputCurrentnessDenial::ExecutionRequest)?
    }

    /// Requires a direct root or dependent demand's retained output lineage at
    /// this exact observation. Receipt-free checkpoint settlements use the same
    /// native authority check; stored fields alone never establish currentness.
    pub fn require_current_output_demand<Query>(
        &self,
        settlement: &crate::application_entry::WorthQueryApplicationOutputDemandSettlement<Query>,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        self.require_current_output_settlements([settlement.retained()])
    }

    fn require_current_output_settlements<'settlement>(
        &self,
        settlements: impl IntoIterator<Item = &'settlement worth_query_execution::facade::primary_graph::WorthQueryOutputDemandSettlement>,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        self.application
            .with_application_advancement(self.scope, |phase| {
                self.require_current_output_settlements_in_advancement(&phase, settlements)
            })
            .map_err(WorthQueryOutputCurrentnessDenial::ExecutionRequest)?
    }
    pub(in crate::application_entry) fn require_current_output_settlements_in_advancement<
        'settlement,
    >(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        settlements: impl IntoIterator<Item = &'settlement worth_query_execution::facade::primary_graph::WorthQueryOutputDemandSettlement>,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        let selected = self
            .application
            .select_application_read_observation(phase, &self.observation)
            .map_err(WorthQueryOutputCurrentnessDenial::Observation)?;
        if selected.product().product_branch() != self.branch {
            return Err(WorthQueryOutputCurrentnessDenial::ForeignBranch);
        }
        selected
            .require_current_output_settlements(phase, settlements)
            .map_err(WorthQueryOutputCurrentnessDenial::Output)
    }
}
