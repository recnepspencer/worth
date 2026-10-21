use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryProducerCommitAuthority;
use super::disclosure::validate_disclosure;
#[cfg(test)]
use super::own_effect_observation;
#[cfg(test)]
use super::own_effect_observation::Stage;
use super::FamilySourceQuery;
use super::FamilySourceValue;
use super::WorthQueryAdmittedOutputDemand;
use super::WorthQueryOutputDemandAdvance;
use super::WorthQueryOutputDemandDenial;
use super::WorthQueryOutputDemandDenialKind;
use super::WorthQueryProducerOutputFamily;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(in crate::domain_computation::primary_graph) use required_wave::CurrentBasis;
pub(in crate::domain_computation::primary_graph) use required_wave::CurrentProof;
mod admission;
mod caller_pass;
use caller_pass::CallerPass;
mod refresh;
mod rejoin;
mod required_fresh;
mod required_wave;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_wave::performed::DecisionInput;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_wave::performed::PerformedMembers;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_wave::performed::SelectedDecisionInput;
mod retained_read;
pub(in crate::domain_computation::primary_graph) use required_wave::MatchedRequiredPredecessors;
pub(in crate::domain_computation::primary_graph) use required_wave::ReboundConsumedOutput;
pub(in crate::domain_computation::primary_graph) use required_wave::ResolvedRequiredPredecessors;
mod selected_program;
mod source_recovery;
mod unavailable;
mod validated;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// A required caller settles first; the request work it leaves over is
    /// spent progressing other dirty required records from the shared queue.
    pub fn advance_output_demand<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

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
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|cause| {
                denial(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "foreign advancement phase",
                )
            })?;
        self.advance_as_caller(demand, |demand, performed| {
            self.advance_output_demand_with_commit_authority(
                &execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
                WorthQueryProducerCommitAuthority::Ordinary,
                performed,
            )
        })
    }

    /// Reenter the admitted demand's frozen source selector, parameters and
    /// scope only if its existing Ready cannot be certified on this call.
    pub fn advance_output_demand_from_retained<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

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
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|cause| {
                denial(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "foreign advancement phase",
                )
            })?;
        self.advance_as_caller(demand, |demand, performed| {
            self.advance_retained_with_commit_authority(
                &execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                WorthQueryProducerCommitAuthority::Ordinary,
                performed,
            )
        })
    }

    /// Reenter the frozen source under the commit authority the demand was
    /// issued with, on the caller's request meter.
    pub(super) fn advance_retained_with_commit_authority<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        commit_authority: WorthQueryProducerCommitAuthority,

        performed: &std::cell::RefCell<PerformedMembers>,
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
        self.advance_output_demand_with_prepared_source(
            execution,
            phase,
            demand,
            principal,
            request_scope,
            delivery_branch,
            |retained| {
                retained_read::read_retained_source::<Schema, Family>(
                    execution,
                    self,
                    retained,
                    principal,
                    request_scope,
                    delivery_branch,
                )
                .map(Some)
            },
            commit_authority,
            performed,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn advance_program_output_demand<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

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
        phase
            .execution_request_for(&self.product_runtime)
            .map_err(|cause| {
                denial(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "the phase belongs to another installed runtime",
                )
            })?;
        self.advance_as_caller(demand, |demand, performed| {
            self.advance_output_demand_with_commit_authority(
                execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
                WorthQueryProducerCommitAuthority::ProgramOutput,
                performed,
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
            &std::cell::RefCell<PerformedMembers>,
        )
            -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let performed = std::cell::RefCell::new(PerformedMembers::start());
        let stop = match advance(demand, &performed) {
            Ok(progress) => {
                demand.settled |= matches!(progress, WorthQueryOutputDemandAdvance::Settled(_));
                return Ok(progress);
            }
            Err(stop) => stop,
        };
        if stop.has_read_cleanup() {
            let diagnostic = stop.clone();
            demand.terminal_caller_denial = Some(stop);
            return Err(diagnostic);
        }
        if matches!(
            stop.kind(),
            WorthQueryOutputDemandDenialKind::ExecutionRequest(_)
        ) && stop.recovery_posture() == super::WorthQueryOutputDemandRecoveryPosture::Terminal
        {
            demand
                .terminal_caller_denial
                .get_or_insert_with(|| stop.clone());
        }
        Err(stop)
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn advance_output_demand_with_commit_authority<
        Family,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        commit_authority: WorthQueryProducerCommitAuthority,

        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let mut disclosure = Some(disclosure);
        self.advance_output_demand_with_prepared_source(
            execution,
            phase,
            demand,
            principal,
            request_scope,
            delivery_branch,
            |_| Ok(disclosure.take()),
            commit_authority,
            performed,
        )
    }
}

pub(super) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<std::borrow::Cow<'static, str>>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}

mod caller_pass_observation;
