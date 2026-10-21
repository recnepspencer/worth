//! Typed caller disclosure, publication, and requested-output progression.
use super::*;
mod own_publication;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

/// What one pass of a caller over its own row answered.
pub(super) enum CallerPass<Query, Value> {
    Answer(WorthQueryOutputDemandAdvance),
    /// The same owner successfully moved its checkpoint, without a new decision input.
    Progressed,
    /// The pass replaced a Ready that no longer proves its output with a row
    /// admitted under the disclosed source. Nothing outside this call has to
    /// happen before that row runs: its stages need the next disclosure.
    Refreshed(super::super::disclosure::ValidatedOutputDisclosure<Query, Value>),
    /// This call's own publication changed a fact its decision read. Reobserve
    /// that publication before refreshing; no outside delivery is required.
    PublishedStale(super::validated::OwnPublication),
}

/// Each pass owns authenticated source custody and any actual publication.
/// An executable edition is sealed only after final demand selection.
enum AdvancePass<Query, Value> {
    Initial,
    Refreshed(super::super::disclosure::ValidatedOutputDisclosure<Query, Value>),
    OwnPublication(super::validated::OwnPublication),
    Final(
        super::super::disclosure::ValidatedOutputDisclosure<Query, Value>,
        super::validated::OwnPublication,
    ),
    /// A completed requested chain permits one final frozen-source disclosure.
    RetryDisclosure,
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
    /// retries after source replacement or upstream completion; one that handed over its
    /// only disclosure reports `Progressed` after a native move and discloses on its next advance.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn advance_output_demand_with_prepared_source<
        Family,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,

        mut disclosure: impl FnMut(
            &Observed<Schema, Family>,
        ) -> Result<
            Option<Disclosed<Schema, Family>>,
            WorthQueryOutputDemandDenial,
        >,
        commit_authority: WorthQueryProducerCommitAuthority,

        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let mut pass = AdvancePass::Initial;
        loop {
            let result = self.advance_caller_pass(
                execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                pass,
                performed,
                commit_authority.clone(),
                &mut disclosure,
            );
            super::super::super::super::request_execution::record_caller_pass_for_test(&result);
            match result {
                Ok(AdvancePassProgress::Answer(answer)) => {
                    if matches!(&answer, WorthQueryOutputDemandAdvance::Settled(_)) {
                        demand.required_continuations.requested.clear();
                    }
                    return Ok(answer);
                }
                Ok(AdvancePassProgress::Continue(next)) => pass = next,
                Err(stop) => {
                    demand.required_continuations.requested.clear();
                    return Err(stop);
                }
            }
        }
    }

    fn advance_caller_pass<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,

        pass: AdvancePass<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        performed: &std::cell::RefCell<required_wave::performed::PerformedMembers>,
        commit_authority: WorthQueryProducerCommitAuthority,
        disclosure: &mut impl FnMut(
            &Observed<Schema, Family>,
        ) -> Result<
            Option<Disclosed<Schema, Family>>,
            WorthQueryOutputDemandDenial,
        >,
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
            AdvancePass::Initial | AdvancePass::Refreshed(_) | AdvancePass::RetryDisclosure => None,
        };
        if !self.own_publication_is_selected(execution, delivery_branch, publication)? {
            return Ok(AdvancePassProgress::Answer(
                WorthQueryOutputDemandAdvance::Pending,
            ));
        }
        if demand.runtime_authority != self.runtime.authority_identity().as_u64()
            || demand.schema_binding != self.installed_schema.binding_identity()
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                Family::IDENTITY,
            ));
        }
        if let Some(stop) = &demand.terminal_caller_denial {
            return Err(stop.clone());
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
        demand.rejoin_refreshed_output(&self.output_demands)?;

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
        demand
            .progression_provenance
            .validate_for_execution(&commit_authority, &entry.edition)?;
        if matches!(
            &pass,
            AdvancePass::Initial | AdvancePass::Refreshed(_) | AdvancePass::RetryDisclosure
        ) && let Some(advance) = required_wave::advance_required_before_caller(
            execution,
            phase,
            self,
            demand,
            principal,
            request_scope,
            delivery_branch,
            &commit_authority,
            performed,
        )? {
            return Ok(AdvancePassProgress::Answer(advance));
        }
        let final_disclosure = matches!(&pass, AdvancePass::RetryDisclosure);
        let requested_authority = commit_authority.clone();
        (|| {
            Ok(match pass {
                AdvancePass::Initial | AdvancePass::RetryDisclosure => {
                    let Some(disclosed) = disclosure(&demand.observed_source)? else {
                        return Ok(AdvancePassProgress::Answer(
                            WorthQueryOutputDemandAdvance::Pending,
                        ));
                    };
                    let disclosed = validate_disclosure(
                        execution,
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
                        execution,
                        phase,
                        demand,
                        principal,
                        request_scope,
                        delivery_branch,
                        disclosed,
                        entry,
                        commit_authority,
                        performed,
                    )?;
                    match answer {
                        CallerPass::Answer(answer) => AdvancePassProgress::Answer(answer),
                        CallerPass::Progressed => {
                            AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Progressed)
                        }
                        CallerPass::Refreshed(_) | CallerPass::PublishedStale(_)
                            if final_disclosure =>
                        {
                            AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Progressed)
                        }
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
                        execution,
                        phase,
                        demand,
                        principal,
                        request_scope,
                        delivery_branch,
                        disclosed,
                        entry,
                        commit_authority,
                        performed,
                    )?;
                    match answer {
                        CallerPass::Answer(answer) => AdvancePassProgress::Answer(answer),
                        CallerPass::Progressed => {
                            AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Progressed)
                        }
                        CallerPass::PublishedStale(receipt) => {
                            AdvancePassProgress::Continue(AdvancePass::OwnPublication(receipt))
                        }
                        CallerPass::Refreshed(_) => {
                            AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Progressed)
                        }
                    }
                }
                AdvancePass::OwnPublication(receipt) => {
                    let Some(disclosed) = disclosure(&demand.observed_source)? else {
                        #[cfg(test)]
                        own_effect_observation::record(Stage::DisclosureExhausted);
                        return Ok(AdvancePassProgress::Answer(
                            WorthQueryOutputDemandAdvance::Progressed,
                        ));
                    };
                    let disclosed = validate_disclosure(
                        execution,
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
                        execution,
                        phase,
                        demand,
                        principal,
                        request_scope,
                        delivery_branch,
                        disclosed,
                        entry,
                        commit_authority,
                        performed,
                    )?;
                    match answer {
                        CallerPass::Answer(answer) => AdvancePassProgress::Answer(answer),
                        CallerPass::Progressed => {
                            AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Progressed)
                        }
                        CallerPass::Refreshed(disclosed) => {
                            AdvancePassProgress::Continue(AdvancePass::Final(disclosed, receipt))
                        }
                        CallerPass::PublishedStale(_) => {
                            AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Progressed)
                        }
                    }
                }
                AdvancePass::Final(disclosed, _receipt) => {
                    let answer = self.advance_validated_output_demand(
                        execution,
                        phase,
                        demand,
                        principal,
                        request_scope,
                        delivery_branch,
                        disclosed,
                        entry,
                        commit_authority,
                        performed,
                    )?;
                    match answer {
                        CallerPass::Answer(answer) => AdvancePassProgress::Answer(answer),
                        CallerPass::Progressed => {
                            AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Progressed)
                        }
                        CallerPass::Refreshed(_) | CallerPass::PublishedStale(_) => {
                            AdvancePassProgress::Answer(WorthQueryOutputDemandAdvance::Progressed)
                        }
                    }
                }
            })
        })()
        .or_else(|mut stop| {
            match required_wave::advance_requested_output(
                execution,
                phase,
                self,
                demand,
                principal,
                request_scope,
                delivery_branch,
                &requested_authority,
                &mut stop,
                performed,
            )? {
                Some(WorthQueryOutputDemandAdvance::Settled(_)) if !final_disclosure => {
                    Ok(AdvancePassProgress::Continue(AdvancePass::RetryDisclosure))
                }
                Some(WorthQueryOutputDemandAdvance::Settled(_)) => Ok(AdvancePassProgress::Answer(
                    WorthQueryOutputDemandAdvance::Progressed,
                )),
                Some(answer) => Ok(AdvancePassProgress::Answer(answer)),
                None => Err(stop),
            }
        })
    }
}
