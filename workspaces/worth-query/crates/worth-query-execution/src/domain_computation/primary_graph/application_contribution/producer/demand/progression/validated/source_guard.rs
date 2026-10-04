//! What a demand does when a commit replaced the source it names. The rule
//! is applied once, before the row is begun, so nothing is claimed.

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    OutputRowStage, WorthQueryAcceptedOutputAuthority as Authority,
};
use crate::domain_computation::primary_graph::WorthQueryObservedSource;

/// Whether the disclosed source is still the one the demand names.
pub(super) enum SourceGuard {
    Named,
    Replaced,
}

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
    ) -> Result<SourceGuard, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        demand.progression_provenance.validate_for_execution(
            commit_authority,
            &entry.edition,
            admission,
        )?;
        demand
            .interest
            .as_ref()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY))?;
        let named = if progression.is_selected() {
            demand.matches_observed_source_admitted(source, admission)?
        } else {
            demand.matches_observed_source(source)
        };
        Ok(if named {
            SourceGuard::Named
        } else {
            SourceGuard::Replaced
        })
    }

    /// A demand that settled keeps following its output: it moves to a row
    /// admitted under the disclosed source, whatever stage its own row had
    /// reached, and so does the holder of a Stable alias or a restored
    /// output. One that never settled does not: its caller demands again.
    ///
    /// That stop is the demand's alone while its row holds a Ready, which
    /// stays for the demands settled on it. A row before its Ready can no
    /// longer publish a current output, so it ends with the stop and gives
    /// its occurrence back to the Ready it replaced. A required wave's
    /// successor is never settled and its row is the wave's own: it ends.
    ///
    /// A row another call drives, or one already stopped, answers as its
    /// begin would.
    pub(super) fn leave_replaced_source<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        selected: bool,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<OwnStages, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let interest = demand
            .interest
            .as_ref()
            .expect("progression entry checked live Interest");
        let end_row = || {
            self.output_demands
                .finish_superseded(interest, Family::IDENTITY)
        };
        if selected {
            return Err(end_row());
        }
        let ready = match self.output_demands.row_stage(interest) {
            OutputRowStage::Driven => {
                return Ok(OwnStages::Answer(WorthQueryOutputDemandAdvance::Pending))
            }
            OutputRowStage::Stopped(denial) => return Err(denial),
            OutputRowStage::Ready(completion) => Some(completion),
            OutputRowStage::BeforeReady => None,
        };
        let held = ready.as_ref().map(|completion| &completion.authority);
        if !demand.settled && !matches!(held, Some(Authority::Stable(_) | Authority::Restored(_))) {
            return Err(match held {
                Some(_) => denial(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    Family::IDENTITY,
                ),
                None => end_row(),
            });
        }
        let (disclosed_value, disclosed_source) = disclosure.into_parts();
        self.refresh_output_demand(
            demand,
            disclosed_value,
            disclosed_source,
            held,
            request_admission,
        )?;
        Ok(OwnStages::Refreshed)
    }
}
