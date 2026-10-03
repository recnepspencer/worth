use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryProducerCommitAuthority;
use super::disclosure::validate_disclosure;
use super::{
    FamilySourceQuery, FamilySourceValue, WorthQueryAdmittedOutputDemand,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

mod admission;
mod refresh;
mod required_fresh;
mod required_wave;
mod retained_read;
pub(in crate::domain_computation::primary_graph) use required_wave::{
    MatchedRequiredPredecessor, ResolvedRequiredPredecessor,
};
pub(super) mod resources;
mod selected_program;
mod source_recovery;
mod validated;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn advance_output_demand<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let mut admission = self
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner
            .read_admission(demand.limits.source_currentness_work());
        self.advance_output_demand_with_commit_authority(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            WorthQueryProducerCommitAuthority::Ordinary,
            &mut admission,
        )
    }

    /// Reenter the admitted demand's frozen source selector, parameters and
    /// scope only if its existing Ready cannot be certified on this call.
    pub fn advance_output_demand_from_retained<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                    Schema,
                    FamilySourceQuery<Schema, Family>,
                > + 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let mut admission = self
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner
            .read_admission(demand.limits.source_currentness_work());
        let limits = demand.limits;
        self.advance_output_demand_with_prepared_source(
            demand,
            principal,
            request_scope,
            delivery_branch,
            |retained, admission| {
                retained_read::read_retained_source::<Schema, Family>(
                    self,
                    retained,
                    principal,
                    request_scope,
                    delivery_branch,
                    limits,
                    admission,
                )
            },
            WorthQueryProducerCommitAuthority::Ordinary,
            &mut admission,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn advance_program_output_demand<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let mut admission = self
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner
            .read_admission(demand.limits.source_currentness_work());
        self.advance_output_demand_with_commit_authority(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            WorthQueryProducerCommitAuthority::ProgramOutput,
            &mut admission,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn advance_output_demand_with_commit_authority<
        Family,
    >(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        commit_authority: WorthQueryProducerCommitAuthority,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        self.advance_output_demand_with_prepared_source(
            demand,
            principal,
            request_scope,
            delivery_branch,
            |_, _| Ok(disclosure),
            commit_authority,
            request_admission,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn advance_output_demand_with_prepared_source<
        Family,
    >(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: impl FnOnce(
            &crate::domain_computation::primary_graph::WorthQueryObservedSource<
                FamilySourceQuery<Schema, Family>,
            >,
            &mut InvalidationEditAdmission,
        ) -> Result<
            crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
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
        if demand.runtime_authority != self.runtime.authority_identity().as_u64()
            || demand.schema_binding != self.installed_schema.binding_identity()
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                Family::IDENTITY,
            ));
        }
        demand
            .interest
            .as_ref()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY))?;
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
            &entry.edition,
            request_admission,
        )? {
            return Ok(advance);
        }
        let disclosure = disclosure(&demand.observed_source, request_admission)?;
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
            commit_authority,
            request_admission,
        )
    }
}

pub(super) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
