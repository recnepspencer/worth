//! One caller advance, with one disclosure retry after its source is replaced
//! or an exactly requested upstream output finishes.

use super::*;

/// What one pass of a caller over its own row answered.
pub(super) enum CallerPass {
    Answer(WorthQueryOutputDemandAdvance),
    /// A source replacement or a completed requested dependency made progress.
    /// Retry the caller's own frozen disclosure; no external work is awaited.
    RetryDisclosure,
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
    /// only disclosure answers `Pending` and discloses on its next advance.
    pub(in crate::domain_computation::primary_graph) fn advance_output_demand_with_prepared_source<
        Family,
    >(
        &self,
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
        for _ in 0..2 {
            if let CallerPass::Answer(advance) = self.advance_caller_pass(
                demand,
                principal,
                request_scope,
                delivery_branch,
                &mut disclosure,
                commit_authority.clone(),
                request_admission,
            )? {
                return Ok(advance);
            }
        }
        // The second pass made progress that needs another source disclosure.
        Ok(WorthQueryOutputDemandAdvance::Pending)
    }

    fn advance_caller_pass<Family>(
        &self,
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
        commit_authority: WorthQueryProducerCommitAuthority,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<CallerPass, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
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
            return Ok(CallerPass::Answer(WorthQueryOutputDemandAdvance::Settled(
                std::sync::Arc::clone(settlement),
            )));
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
        if let Some(advance) = required_wave::advance_required_before_caller(
            self,
            demand,
            principal,
            request_scope,
            delivery_branch,
            &commit_authority,
            request_admission,
        )? {
            return Ok(CallerPass::Answer(advance));
        }
        let Some(disclosure) = disclosure(&demand.observed_source, request_admission)? else {
            return Ok(CallerPass::Answer(WorthQueryOutputDemandAdvance::Pending));
        };
        let disclosure = validate_disclosure(
            self,
            demand,
            principal,
            request_scope,
            delivery_branch,
            matches!(
                commit_authority,
                WorthQueryProducerCommitAuthority::ProgramOutput
            ),
            disclosure,
            entry.edition,
        )?;
        self.advance_validated_output_demand(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            entry,
            commit_authority.clone(),
            request_admission,
        )
        .or_else(|mut stop| {
            match required_wave::advance_requested_output(
                self,
                demand,
                principal,
                request_scope,
                delivery_branch,
                &commit_authority,
                &mut stop,
                request_admission,
            )? {
                Some(WorthQueryOutputDemandAdvance::Settled(_)) => Ok(CallerPass::RetryDisclosure),
                Some(advance) => Ok(CallerPass::Answer(advance)),
                None => Err(stop),
            }
        })
    }
}
