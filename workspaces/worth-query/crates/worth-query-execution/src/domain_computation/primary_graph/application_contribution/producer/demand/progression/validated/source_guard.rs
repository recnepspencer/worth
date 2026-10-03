//! Preserve the ordinary source guard after admission. The selected source
//! was already checked before the first registry transition.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn prepare_progression_entry<Family>(
        &self,
        demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
        source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
        commit_authority: &WorthQueryProducerCommitAuthority,
        entry: &InstalledProducerProvider<Schema>,
        progression: &ScheduleProgression<'_, '_, Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        demand.progression_provenance.validate_for_execution(
            commit_authority,
            &entry.edition,
            admission,
        )?;
        let interest = demand
            .interest
            .as_ref()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY))?;
        if progression.is_selected()
            && !demand.matches_observed_source_admitted(source, admission)?
        {
            return Err(self
                .output_demands
                .finish_superseded(interest, Family::IDENTITY));
        }
        Ok(())
    }

    pub(super) fn require_progression_source<Family>(
        &self,
        interest: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest,
        demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
        source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
        progression: &ScheduleProgression<'_, '_, Schema>,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        if !progression.source_matches(demand, source) {
            return Err(self
                .output_demands
                .finish_superseded(interest, Family::IDENTITY));
        }
        Ok(())
    }
}
