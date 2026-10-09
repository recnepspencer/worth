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
mod caller_custody;
mod caller_pass;
use caller_pass::CallerPass;
mod refresh;
mod rejoin;
mod required_fresh;
mod required_wave;
mod retained_read;
pub(in crate::domain_computation::primary_graph) use required_wave::{
    MatchedRequiredPredecessors, ResolvedRequiredPredecessors,
};
pub(super) mod resources;
mod selected_program;
mod source_recovery;
mod unavailable;
mod validated;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Framework preparation and every producer one advance runs share this
    /// structurally bounded request meter. Neither demand lane funds it:
    /// currentness verification spends the source-currentness lane on its own
    /// meter, and the producer-work lane bounds each producer's declared work
    /// and source reads.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn demand_request_admission(
        &self,
    ) -> InvalidationEditAdmission {
        self.primary_provider
            .graph
            .source_owner
            .invalidation_owner
            .request_admission()
    }

    /// A required caller settles first; the request work it leaves over is
    /// spent progressing other dirty required records from the shared queue.
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
        self.advance_as_caller(demand, |demand, admission| {
            self.advance_output_demand_with_commit_authority(
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
                WorthQueryProducerCommitAuthority::Ordinary,
                admission,
            )
        })
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
        self.advance_as_caller(demand, |demand, admission| {
            self.advance_retained_with_commit_authority(
                demand,
                principal,
                request_scope,
                delivery_branch,
                WorthQueryProducerCommitAuthority::Ordinary,
                admission,
            )
        })
    }

    /// Reenter the frozen source under the commit authority the demand was
    /// issued with, on the caller's request meter.
    pub(super) fn advance_retained_with_commit_authority<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        commit_authority: WorthQueryProducerCommitAuthority,
        admission: &mut InvalidationEditAdmission,
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
                .map(Some)
            },
            commit_authority,
            admission,
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
        self.advance_as_caller(demand, |demand, admission| {
            self.advance_output_demand_with_commit_authority(
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
                WorthQueryProducerCommitAuthority::ProgramOutput,
                admission,
            )
        })
    }

    /// One advance a caller starts, on a fresh request meter. A custody
    /// refusal reaches the caller as that caller meets it; advances nested
    /// in this one keep the rows' own stops.
    pub(super) fn advance_as_caller<Family>(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        advance: impl FnOnce(
            &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
            &mut InvalidationEditAdmission,
        )
            -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let mut admission = self.demand_request_admission();
        let stop = match advance(demand, &mut admission) {
            Ok(progress) => {
                demand.settled |= matches!(progress, WorthQueryOutputDemandAdvance::Settled(_));
                return Ok(progress);
            }
            Err(stop) => stop,
        };
        Err(self.caller_custody_stop(demand, stop, &mut admission))
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
        let mut disclosure = Some(disclosure);
        self.advance_output_demand_with_prepared_source(
            demand,
            principal,
            request_scope,
            delivery_branch,
            |_, _| Ok(disclosure.take()),
            commit_authority,
            request_admission,
        )
    }
}

pub(super) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<std::borrow::Cow<'static, str>>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
