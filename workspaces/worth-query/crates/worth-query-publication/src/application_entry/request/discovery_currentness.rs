//! Selected-image handoff for complete original discovery and output evidence.
use super::super::*;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::{
    facade::{
        application_contribution::WorthQueryAdvancementPhase,
        primary_graph::WorthQueryApplicationOutputDemandSource,
    },
    publication_boundary::{
        discovery_currentness::{
            WorthQueryRequiredSetCurrentnessDenial, WorthQueryRetainedQueryDependencies,
        },
        program_publication_access,
    },
};
impl<'a, 'p, 's, Schema: ApplicationSchema>
    WorthQueryApplicationRetainedRequest<'a, 'p, 's, Schema>
{
    pub(in crate::application_entry) fn retain_query_dependencies<Query, Value>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        source: &WorthQueryApplicationOutputDemandSource<Query, Value>,
    ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredOutputPreparationDenial>
    {
        self.application
            .select_application_read_observation(phase, &self.observation)
            .map_err(WorthQueryRequiredOutputPreparationDenial::ReadObservation)?
            .retain_query_dependencies(phase, &program_publication_access(), source)
            .map_err(WorthQueryRequiredOutputPreparationDenial::Dependencies)
    }
    pub(in crate::application_entry) fn combine_query_dependencies<'d, I>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        parts: I,
    ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredOutputPreparationDenial>
    where
        I: Iterator<Item = &'d WorthQueryRetainedQueryDependencies> + Clone,
    {
        self.application
            .select_application_read_observation(phase, &self.observation)
            .map_err(WorthQueryRequiredOutputPreparationDenial::ReadObservation)?
            .combine_query_dependencies(phase, &program_publication_access(), parts)
            .map_err(WorthQueryRequiredOutputPreparationDenial::Dependencies)
    }
    pub(in crate::application_entry) fn require_current_required_set_in_advancement<'d>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        settlements: impl IntoIterator<Item=&'d worth_query_execution::facade::primary_graph::WorthQueryOutputDemandSettlement>,
        dependencies: &WorthQueryRetainedQueryDependencies,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        let selected = self
            .application
            .select_application_read_observation(phase, &self.observation)
            .map_err(WorthQueryOutputCurrentnessDenial::Observation)?;
        if selected.product().product_branch() != self.branch {
            return Err(WorthQueryOutputCurrentnessDenial::ForeignBranch);
        }
        selected
            .require_current_required_set(
                phase,
                &program_publication_access(),
                self.scope,
                settlements,
                dependencies,
            )
            .map_err(|cause| match cause {
                WorthQueryRequiredSetCurrentnessDenial::Source(cause) => {
                    WorthQueryOutputCurrentnessDenial::Source(cause)
                }
                WorthQueryRequiredSetCurrentnessDenial::Output(cause) => {
                    WorthQueryOutputCurrentnessDenial::Output(cause)
                }
            })
    }
    /// Checks the original complete discovery consumption and every actually
    /// retained root and dependent output at this request's selected image.
    /// Native retired roots require no invented settlement. Their discovery
    /// membership remains required, including originally empty sets.
    pub fn require_current_discovered_program_outputs<Q, D>(
        &self,
        settlement: &WorthQueryDiscoveredProgramOutputSettlement<Q, D>,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        self.application
            .with_application_advancement(self.scope, |phase| {
                self.require_current_discovered_program_outputs_in_advancement(&phase, settlement)
            })
            .map_err(WorthQueryOutputCurrentnessDenial::ExecutionRequest)?
    }
    pub(in crate::application_entry) fn require_current_discovered_program_outputs_in_advancement<
        Q,
        D,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        settlement: &WorthQueryDiscoveredProgramOutputSettlement<Q, D>,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        self.require_current_required_set_in_advancement(
            phase,
            settlement.retained_settlements(),
            settlement.dependencies(),
        )
    }
    pub(in crate::application_entry) fn require_current_program_output_in_advancement<Q>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        settlement: &WorthQueryApplicationProgramOutputSettlement<Q>,
    ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
        match settlement.dependencies.as_ref() {
            Some(dependencies) => self.require_current_required_set_in_advancement(
                phase,
                settlement.retained_settlements(),
                dependencies,
            ),
            None => self.require_current_output_settlements_in_advancement(
                phase,
                settlement.retained_settlements(),
            ),
        }
    }
}
