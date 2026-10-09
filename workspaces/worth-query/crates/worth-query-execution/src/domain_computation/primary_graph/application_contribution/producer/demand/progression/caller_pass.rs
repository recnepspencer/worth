//! One caller advance over its own row: a pass, and a second one when the
//! first replaced its Ready with a row admitted under the disclosed source.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

/// What one pass of a caller over its own row answered.
pub(super) enum CallerPass<Query, Value> {
    Answer(WorthQueryOutputDemandAdvance),
    /// The pass replaced a Ready that no longer proves its output with a row
    /// admitted under the disclosed source. Nothing outside this call has to
    /// happen before that row runs: its stages need the next disclosure.
    Refreshed(super::super::disclosure::ValidatedOutputDisclosure<Query, Value>),
    /// This call's own publication changed a fact its decision read. Reobserve
    /// that publication before refreshing; no outside delivery is required.
    PublishedStale(super::validated::OwnPublication),
}

/// Each pass owns its source and, for a reobservation, the actual publication.
enum AdvancePass<Query, Value> {
    Initial,
    Refreshed(super::super::disclosure::ValidatedOutputDisclosure<Query, Value>),
    OwnPublication(super::validated::OwnPublication),
    Final(
        super::super::disclosure::ValidatedOutputDisclosure<Query, Value>,
        super::validated::OwnPublication,
    ),
}
enum AdvancePassProgress<Query, Value> {
    Answer(WorthQueryOutputDemandAdvance),
    Continue(AdvancePass<Query, Value>),
}

type Disclosed<Schema, Family> =
    crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
        FamilySourceQuery<Schema, Family>,
        FamilySourceValue<Schema, Family>,
    >;
type Observed<Schema, Family> = crate::domain_computation::primary_graph::WorthQueryObservedSource<
    FamilySourceQuery<Schema, Family>,
>;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// `disclosure` answers `None` once the caller has no further source to
    /// disclose in this call. A caller that reads its retained source again
    /// settles a refreshed row in the same advance; one that handed over its
    /// only disclosure answers `Pending` and discloses on its next advance.
    pub(in crate::domain_computation::primary_graph) fn advance_output_demand_with_prepared_source<
        Family,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        mut disclosure: impl FnMut(
            &Observed<Schema, Family>,
            &mut InvalidationEditAdmission,
        ) -> Result<
            Option<Disclosed<Schema, Family>>,
            WorthQueryOutputDemandDenial,
        >,
        commit_authority: WorthQueryProducerCommitAuthority,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let mut pass = AdvancePass::Initial;
        loop {
            let result = self.advance_caller_pass(
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                &mut disclosure,
                pass,
                commit_authority.clone(),
                request_admission,
            );
            #[cfg(feature = "test-query-execution-observer")]
            super::super::super::super::request_execution::record_caller_pass(&result);
            match result? {
                AdvancePassProgress::Answer(answer) => return Ok(answer),
                AdvancePassProgress::Continue(next) => pass = next,
            }
        }
    }

    fn advance_caller_pass<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: &mut impl FnMut(
            &Observed<Schema, Family>,
            &mut InvalidationEditAdmission,
        ) -> Result<
            Option<Disclosed<Schema, Family>>,
            WorthQueryOutputDemandDenial,
        >,
        pass: AdvancePass<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        commit_authority: WorthQueryProducerCommitAuthority,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<
        AdvancePassProgress<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        WorthQueryOutputDemandDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let publication = match &pass {
            AdvancePass::OwnPublication(receipt) | AdvancePass::Final(_, receipt) => Some(receipt),
            AdvancePass::Initial | AdvancePass::Refreshed(_) => None,
        };
        if let Some(publication) = publication {
            #[cfg(feature = "test-query-execution-observer")]
            super::caller_pass_observation::run_interleaving();
            let selected = self.on_branch(delivery_branch).select().map_err(|stop| {
                WorthQueryOutputDemandDenial::product_selection(
                    stop,
                    "own publication continuation",
                )
            })?;
            if !publication
                .0
                .is_selected_at(selected.product().observation())
            {
                return Ok(AdvancePassProgress::Answer(
                    WorthQueryOutputDemandAdvance::Pending,
                ));
            }
        }
        if demand.runtime_authority != self.runtime.authority_identity().as_u64()
            || demand.schema_binding != self.installed_schema.binding_identity()
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                Family::IDENTITY,
            ));
        }
        if let Some(settlement) = demand.settled_at_observation.as_ref() {
            // No commit after its observation applies to this demand.
            return Ok(AdvancePassProgress::Answer(
                WorthQueryOutputDemandAdvance::Settled(std::sync::Arc::clone(settlement)),
            ));
        }
        demand
            .interest
            .as_ref()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY))?;
        demand.rejoin_refreshed_output(&self.output_demands, request_admission)?;
        request_admission
            .charge_external_work((std::mem::size_of_val(&demand.installed_entry) + 1) as u64)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        let installed_entry = std::sync::Arc::clone(&demand.installed_entry);
        let entry = installed_entry.as_ref();
        if !entry
            .declaration
            .applicability
            .contains(&demand.selected.applicability)
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                format!(
                    "{}: admitted applicability {:?} is no longer installed",
                    demand.selected.identity, demand.selected.applicability
                ),
            ));
        }
        // A required successor carries the mode and installed edition that
        // issued it. Check those before a Current-only required wave can
        // acknowledge work without entering the validated execution core.
        demand.progression_provenance.validate_for_execution(
            &commit_authority,
            &entry.edition,
            request_admission,
        )?;
        if matches!(&pass, AdvancePass::Initial | AdvancePass::Refreshed(_)) {
            if let Some(advance) = required_wave::advance_required_before_caller(
                phase,
                self,
                demand,
                principal,
                request_scope,
                delivery_branch,
                &commit_authority,
                &entry.edition,
                request_admission,
            )? {
                return Ok(AdvancePassProgress::Answer(advance));
            }
        }
        Ok(match pass {
            AdvancePass::Initial => {
                let Some(disclosed) = disclosure(&demand.observed_source, request_admission)?
                else {
                    return Ok(AdvancePassProgress::Answer(
                        WorthQueryOutputDemandAdvance::Pending,
                    ));
                };
                let disclosed = validate_disclosure(
                    self,
                    demand,
                    principal,
                    request_scope,
                    delivery_branch,
                    matches!(
                        commit_authority,
                        WorthQueryProducerCommitAuthority::ProgramOutput
                    ),
                    disclosed,
                    entry.edition,
                )?;
                let answer = self.advance_validated_output_demand(
                    phase,
                    demand,
                    principal,
                    request_scope,
                    delivery_branch,
                    disclosed,
                    entry,
                    commit_authority,
                    request_admission,
                )?;
                match answer {
                    CallerPass::Answer(answer) => AdvancePassProgress::Answer(answer),
                    CallerPass::Refreshed(disclosed) => {
                        AdvancePassProgress::Continue(AdvancePass::Refreshed(disclosed))
                    }
                    CallerPass::PublishedStale(receipt) => {
                        AdvancePassProgress::Continue(AdvancePass::OwnPublication(receipt))
                    }
                }
            }
            AdvancePass::Refreshed(disclosed) => {
                let answer = self.advance_validated_output_demand(
                    phase,
                    demand,
                    principal,
                    request_scope,
                    delivery_branch,
                    disclosed,
                    entry,
                    commit_authority,
                    request_admission,
                )?;
                match answer {
                    CallerPass::Answer(answer) => AdvancePassProgress::Answer(answer),
                    CallerPass::PublishedStale(receipt) => {
                        AdvancePassProgress::Continue(AdvancePass::OwnPublication(receipt))
                    }
                    CallerPass::Refreshed(_) => {
                        AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Pending)
                    }
                }
            }
            AdvancePass::OwnPublication(receipt) => {
                let Some(disclosed) = disclosure(&demand.observed_source, request_admission)?
                else {
                    return Ok(AdvancePassProgress::Answer(
                        WorthQueryOutputDemandAdvance::Pending,
                    ));
                };
                let disclosed = validate_disclosure(
                    self,
                    demand,
                    principal,
                    request_scope,
                    delivery_branch,
                    matches!(
                        commit_authority,
                        WorthQueryProducerCommitAuthority::ProgramOutput
                    ),
                    disclosed,
                    entry.edition,
                )?;
                let answer = self.advance_validated_output_demand(
                    phase,
                    demand,
                    principal,
                    request_scope,
                    delivery_branch,
                    disclosed,
                    entry,
                    commit_authority,
                    request_admission,
                )?;
                match answer {
                    CallerPass::Answer(answer) => AdvancePassProgress::Answer(answer),
                    CallerPass::Refreshed(disclosed) => {
                        AdvancePassProgress::Continue(AdvancePass::Final(disclosed, receipt))
                    }
                    CallerPass::PublishedStale(_) => {
                        AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Pending)
                    }
                }
            }
            AdvancePass::Final(disclosed, _receipt) => {
                let answer = self.advance_validated_output_demand(
                    phase,
                    demand,
                    principal,
                    request_scope,
                    delivery_branch,
                    disclosed,
                    entry,
                    commit_authority,
                    request_admission,
                )?;
                match answer {
                    CallerPass::Answer(answer) => AdvancePassProgress::Answer(answer),
                    CallerPass::Refreshed(_) | CallerPass::PublishedStale(_) => {
                        AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Pending)
                    }
                }
            }
        })
    }
}
