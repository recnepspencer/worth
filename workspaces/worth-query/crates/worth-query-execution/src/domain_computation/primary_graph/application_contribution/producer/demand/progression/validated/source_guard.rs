//! What a demand does when a commit replaced the source it names. The rule
//! is applied once, before the row is begun, so nothing is claimed.

use crate::domain_computation::primary_graph::application_contribution::producer::demand::progression::refresh::RefreshParentOps;
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::OutputRowStage;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority as Authority;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerCommitAuthority;

/// Whether the disclosed source is still the one the demand names.
pub(super) enum SourceGuard {
    Named,
    Replaced,
}

pub(super) trait SourceGuardParentOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn released_predecessor_needs_source<Family>(
        &self,
        demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
        source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    ) -> Result<bool, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let interest = demand
            .interest
            .as_ref()
            .expect("progression entry checked live Interest");
        // SourceEpoch compares fixed owner-issued source coordinates and their
        // sealed meaning identity. This neither reads the graph nor establishes
        // equality/currentness for any retained output.

        let epoch = source.output_source_epoch().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "released source identity",
            )
        })?;
        if interest.key().source_epoch().same_semantic_source(&epoch) {
            return Ok(false);
        }
        Ok(matches!(
            self.output_production()
                .demands()
                .row_stage_admitted(interest)?,
            OutputRowStage::BeforeReady
        ))
    }

fn prepare_progression_entry<Family>(
        &self,
        demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
        source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
        commit_authority: &WorthQueryProducerCommitAuthority,
        entry: &InstalledProducerProvider<Schema>,
        progression: &ScheduleProgression<'_, '_, '_, Schema>,
    ) -> Result<SourceGuard, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        demand
            .progression_provenance
            .validate_for_execution(commit_authority, &entry.edition)?;
        demand
            .interest
            .as_ref()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY))?;
        let named = if progression.is_selected() {
            demand.matches_observed_source_admitted(source)?
        } else {
            demand.matches_observed_source(source)
        };
        Ok(if named {
            SourceGuard::Named
        } else {
            SourceGuard::Replaced
        })
    }

fn leave_replaced_source<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        selected: bool,
    ) -> Result<
        OwnStages<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        WorthQueryOutputDemandDenial,
    >
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
            self.output_production()
                .demands()
                .finish_superseded(interest, Family::IDENTITY)
        };
        if selected {
            return Err(end_row()?);
        }
        let ready = match self.output_production().demands().row_stage(interest) {
            OutputRowStage::Driven => {
                return Ok(OwnStages::Answer(WorthQueryOutputDemandAdvance::Pending))
            }
            OutputRowStage::Stopped(denial)
                if !demand.settled
                    || denial.kind() != WorthQueryOutputDemandDenialKind::Superseded =>
            {
                return Err(denial)
            }
            OutputRowStage::Ready(completion) => Some(completion),
            OutputRowStage::Stopped(_)
            | OutputRowStage::BeforeReady
            | OutputRowStage::Published
            | OutputRowStage::PublicationPending => None,
        };
        let held = ready.as_ref().map(|completion| &completion.authority);
        // A changed source at the Ready's own publication is born stale:
        // this demand must refresh to the lasting result of that source.
        // An unrelated later publication still supersedes an unsettled caller.
        let own_publication = matches!(held, Some(Authority::Committed(receipt))
            if disclosure.source().selected_product_commit()
                == Some(receipt.committed_product_publication().composite_commit()));
        if !demand.settled
            && !own_publication
            && !matches!(held, Some(Authority::Stable(_) | Authority::Restored(_)))
        {
            return Err(match held {
                Some(_) => denial(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    Family::IDENTITY,
                ),
                None => end_row()?,
            });
        }

        if let Some(completion) = ready.as_ref() {
            self.refresh_ready_output_demand(
                execution,
                demand,
                disclosure.value(),
                disclosure.source().clone(),
                completion,
            )?;
        } else {
            self.refresh_output_demand(
                execution,
                demand,
                disclosure.value(),
                disclosure.source().clone(),
                None,
            )?;
        }
        // Carry source custody unchanged; the admitted successor supplies its seal.
        Ok(OwnStages::Refreshed(disclosure))
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> SourceGuardParentOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}
