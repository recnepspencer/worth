use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;

use super::super::ProducerInvocationAuthority;
use super::super::WorthQueryProducerCommitAuthority;
use super::disclosure::validate_disclosure;
use super::own_effect_observation;
use super::own_effect_observation::Stage;
use super::FamilySourceQuery;
use super::FamilySourceValue;
use super::WorthQueryAdmittedOutputDemand;
use super::WorthQueryOutputDemandAdvance;
use super::WorthQueryProducerOutputFamily;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind;


pub(in crate::domain_computation::primary_graph) use required_wave::CurrentBasis;
pub(in crate::domain_computation::primary_graph) use required_wave::CurrentProof;
mod admission;
mod caller_pass;
use caller_pass::CallerPass;
mod refresh;
mod rejoin;
mod required_fresh;
pub(crate) mod required_wave;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_wave::performed::DecisionInput;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_wave::performed::PerformedMembers;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_wave::performed::SelectedDecisionInput;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) mod retained_read;
pub(in crate::domain_computation::primary_graph) use required_wave::MatchedRequiredPredecessors;
pub(in crate::domain_computation::primary_graph) use required_wave::ReboundConsumedOutput;
pub(in crate::domain_computation::primary_graph) use required_wave::ResolvedRequiredPredecessors;
mod selected_program;
mod source_recovery;
mod unavailable;
mod validated;

pub trait ProgressionOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn advance_output_demand<Family>(
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
            .execution_request_for(self.product_runtime())
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
                WorthQueryProducerCommitAuthority::Ordinary.into(),
                performed,
            )
        })
    }

fn advance_output_demand_from_retained<Family>(
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
            .execution_request_for(self.product_runtime())
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
                WorthQueryProducerCommitAuthority::Ordinary.into(),
                performed,
            )
        })
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> ProgressionOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}

pub(super) trait ProgressionParentOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn advance_retained_with_commit_authority<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        commit_authority: ProducerInvocationAuthority<'_>,

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
        let history = commit_authority.history_control();
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
                    self.application_contribution_host(),
                    retained,
                    principal,
                    request_scope,
                    delivery_branch,
                    history,
                )
                .map(Some)
            },
            commit_authority,
            performed,
        )
    }

fn advance_as_caller<Family>(
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
        ) && stop.recovery_posture() == crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandRecoveryPosture::Terminal
        {
            demand
                .terminal_caller_denial
                .get_or_insert_with(|| stop.clone());
        }
        Err(stop)
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> ProgressionParentOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}

pub(in crate::domain_computation::primary_graph) trait ProgressionScopedOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn advance_program_output_demand<Family>(
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
            .execution_request_for(self.product_runtime())
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
                WorthQueryProducerCommitAuthority::ProgramOutput.into(),
                performed,
            )
        })
    }

fn advance_history_output_demand<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        history: crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation<'_>,
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
        self.advance_as_caller(demand, |demand, performed| {
            self.advance_retained_with_commit_authority(
                execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                ProducerInvocationAuthority::history(
                    WorthQueryProducerCommitAuthority::ProgramOutput,
                    history,
                ),
                performed,
            )
        })
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> ProgressionScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) trait DemandProgressionScopedOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn advance_output_demand_with_commit_authority<
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
        commit_authority: ProducerInvocationAuthority<'_>,

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

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> DemandProgressionScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}

pub fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<std::borrow::Cow<'static, str>>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}

pub(in crate::domain_computation::primary_graph) mod caller_pass_observation;

mod court_doors_for_test;
pub use admission::AdmissionOps;
pub(in crate::domain_computation) use admission::AdmissionScopedOps;
pub use admission::AdmissionDoorsOps;
pub(in crate::domain_computation) use admission::DependentScopedOps;
pub use admission::RetainedObservationOps;
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) use admission::SourceAdmissionScopedOps;
pub(in crate::domain_computation::primary_graph) use admission::SourceCustodyScopedOps;
pub use admission::SourceCustodyOps;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use caller_pass::CallerPassScopedOps;
pub use caller_pass_observation::CallerPassObservationOps;
pub use court_doors_for_test::ProgressionDoorsOps;
pub use refresh::RecoveryRefreshObservationOps;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_fresh::RequiredFreshScopedOps;
pub use selected_program::SelectedProgramOps;
pub(in crate::domain_computation::primary_graph) use source_recovery::SourceRecoveryScopedOps;
pub(in crate::domain_computation::primary_graph) use unavailable::UnavailableScopedOps;
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) use validated::EntryScopedOps;
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) use validated::ExecutionPreparationScopedOps;
