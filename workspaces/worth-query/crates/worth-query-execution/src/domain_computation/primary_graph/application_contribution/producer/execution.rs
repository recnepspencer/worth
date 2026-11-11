use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
mod contact_observation;
mod current_control;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use current_control::{select_history_control, select_invocation_control};
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationInputScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolution;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRole;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use super::ProducerInvocationAuthority;
use super::ProducerSourceBinding as SourceBinding;
use super::ProducerSourceQuery as SourceQuery;
use super::ProducerSourceValue as SourceValue;
use super::WorthQueryApplicationProducerBinding;
use super::WorthQueryApplicationProducerProvider;
use super::WorthQueryProducerCommitAuthority;
use crate::basis::WorthQueryProductBranch;

use crate::domain_computation::primary_graph::RequiredOutputDemandContext;
use crate::domain_computation::primary_graph::RequiredOutputExecution;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

use super::demand::WorthQueryOutputDemandDenial;
use super::demand::WorthQueryOutputDemandDenialKind;

mod admitted;
mod coupled_adoption;
mod coupled_preparation;
pub(in crate::domain_computation::primary_graph::application_contribution) use coupled_preparation::preparation_denial as coupled_preparation_denial;
mod coupled;
mod coupled_source;
pub(in crate::domain_computation::primary_graph::application_contribution) use coupled::InstalledCoupledProducerExecutor;
pub(in crate::domain_computation::primary_graph::application_contribution) use coupled::PreparedCoupledWriter;
pub(in crate::domain_computation::primary_graph::application_contribution) use coupled_source::{
    CoupledSourceRead, CoupledSourceReadStop, ErasedAppliedProducerSource,
};
pub(crate) mod denial;
mod input_cutoff;
mod input_reuse;
mod installed;
pub(crate) mod outcome;
mod post_authorization;
pub(crate) mod prepared_plan;
pub(in crate::domain_computation::primary_graph) mod unpublished;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use prepared_plan::canonical_apply;
mod required_cue;
mod selected;
mod selected_public;
mod source_readmission;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use source_readmission::ready_currentness_denial;
use super::demand::disclosure::FreshOutputDisclosure;
use super::demand::disclosure::ValidatedOutputDisclosure;
use super::demand::disclosure::ValidatedProducerInput;
use admitted::execute_fresh;
pub(super) use denial::ProducerExecutionStop;
use denial::denial;
use denial::execution_failed;
use denial::failed;
use denial::identity_unavailable;
use denial::principal_rejected;
use denial::query_admission_denied;
use denial::query_execution_denied;
use denial::request_authority_denied;
use denial::scope_rejected;
use selected::InstalledSelectedProducerExecutor;

use input_reuse::prepared_key;
use input_reuse::require_selected_program;
pub(super) use installed::TypedInstalledProducer;
use outcome::PreparedProducerExecutionOutcome;
pub(super) use outcome::ProducerCommitReceipt;
pub(super) use outcome::ProducerExecutionOutcome;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_cue::RequiredCueProgress;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_cue::FrozenCurrentInspection;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_cue::FrozenInspection;
pub(super) use outcome::commit_receipt;
use outcome::completed_handler;
use source_readmission::readmit_source;

type Operation<Schema, Binding> =
    <Binding as WorthQueryApplicationProducerBinding<Schema>>::Operation;

pub(super) trait InstalledProducerExecutor<Schema>:
    InstalledSelectedProducerExecutor<Schema> + InstalledCoupledProducerExecutor<Schema> + Send + Sync
{
    fn dependency_scopes(
        &self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<super::super::authored_edit::ProducerDependencyScopes, WorthQueryOutputDemandDenial>;
    fn supports_history_group(&self) -> bool;

    fn recover_publication(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        invocation: &ProducerInvocationAuthority<'_>,
        edition: super::InstalledProducerEdition,
        owner: &mut unpublished::PendingProducerPublication,
    ) -> Result<Option<unpublished::RecoveredProducerPublication>, WorthQueryOutputDemandDenial>;

    fn execute(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        required_output: RequiredOutputExecution,
        input: ValidatedProducerInput<'_>,
        successor_of: Option<[u8; 32]>,
        commit_authority: ProducerInvocationAuthority<'_>,
        edition: super::InstalledProducerEdition,
        producer_contacts: &mut usize,
    ) -> Result<PreparedProducerExecutionOutcome, ProducerExecutionStop>;

    fn readiness_record(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        WorthQueryOutputDemandDenial,
    >;
}

impl<Schema, Binding> InstalledProducerExecutor<Schema> for TypedInstalledProducer<Schema, Binding>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    Binding::Provider: WorthQueryApplicationProducerProvider<Schema, Binding>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                Schema,
                SourceQuery<Schema, Binding>,
            > + 'static,
    SourceQuery<Schema, Binding>: 'static,
{
    fn dependency_scopes(
        &self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<super::super::authored_edit::ProducerDependencyScopes, WorthQueryOutputDemandDenial>
    {
        let mut queries = super::super::authored_edit::managed_reads(
            application,
            self.mutation.operation().contracts().managed_index_reads(),
        )?;
        queries.push(super::super::authored_edit::DeclaredQueryRead {
            identity: self.source_query.identity().into(),
            contract: self.source_query.query().read_graph().clone(),
        });
        Ok(super::super::authored_edit::ProducerDependencyScopes { identity: Binding::IDENTITY.into(), family: <Binding::OutputFamily as worth_query_declaration::facade::application_program::WorthQueryProducerOutputFamily<Schema>>::IDENTITY, contract: self.mutation.operation().contracts().read_touch_overlap().clone(), queries })
    }
    fn supports_history_group(&self) -> bool {
        use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
        !Operation::<Schema, Binding>::WORKFLOW_CONTROL
            && !Operation::<Schema, Binding>::REQUIRES_WORKFLOW_AUTHORITY
            && !Operation::<Schema, Binding>::workflow_execution_requirements().is_applicable()
            && !self
                .mutation
                .operation()
                .contracts()
                .external_effect()
                .is_declared()
    }

    fn recover_publication(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        invocation: &ProducerInvocationAuthority<'_>,
        edition: super::InstalledProducerEdition,
        owner: &mut unpublished::PendingProducerPublication,
    ) -> Result<Option<unpublished::RecoveredProducerPublication>, WorthQueryOutputDemandDenial>
    {
        owner.state_mut().progress::<Schema, Binding>(
            phase, self, runtime, principal, request, invocation, edition,
        )
    }

    fn readiness_record(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        WorthQueryOutputDemandDenial,
    > {
        let entity = receipt
            .output_correspondence()
            .entity_for_binding_role::<Binding::Operation>(
                <Binding::OutputRole as WorthQueryApplicationOutputRole>::NAME,
            )
            .map_err(|error| failed(Binding::IDENTITY, error))?;
        Ok(
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                entity.partition_id.0,
                entity.local_slot.0,
                entity.generation.0,
            ),
        )
    }

    fn execute(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        required_output: RequiredOutputExecution,
        input: ValidatedProducerInput<'_>,
        successor_of: Option<[u8; 32]>,
        commit_authority: ProducerInvocationAuthority<'_>,
        edition: super::InstalledProducerEdition,
        producer_contacts: &mut usize,
    ) -> Result<PreparedProducerExecutionOutcome, ProducerExecutionStop> {
        let execution = phase
            .execution_request_for(&runtime.runtime.product_runtime)
            .map_err(|cause| {
                denial(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "producer advancement phase",
                )
            })?;
        let (required_output, ready_backing) = required_output.into_parts();
        if !edition.admits_binding::<Schema, Binding>() {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Binding::IDENTITY,
            )
            .into());
        }
        let input = input
            .take::<SourceQuery<Schema, Binding>, SourceValue<Schema, Binding>>(edition)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Binding::IDENTITY,
                )
            })?;
        let outcome = match input {
            ValidatedOutputDisclosure::Fresh(proof) => execute_fresh::<Schema, Binding>(
                &execution,
                phase,
                self,
                runtime,
                principal,
                request_scope,
                branch,
                required_output,
                &self.provider,
                proof,
                successor_of,
                commit_authority,
                edition,
                producer_contacts,
            ),
            ValidatedOutputDisclosure::RetainedProgram(proof) => {
                if matches!(
                    commit_authority.mode(),
                    WorthQueryProducerCommitAuthority::Ordinary
                ) || !proof.admits(principal, request_scope)
                {
                    return Err(denial(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        Binding::IDENTITY,
                    )
                    .into());
                }
                let fresh = readmit_source::<Schema, Binding>(
                    &execution,
                    runtime,
                    principal,
                    request_scope,
                    branch,
                    proof.source(),
                    edition,
                )?;
                execute_fresh::<Schema, Binding>(
                    &execution,
                    phase,
                    self,
                    runtime,
                    principal,
                    request_scope,
                    branch,
                    required_output,
                    &self.provider,
                    fresh,
                    successor_of,
                    commit_authority,
                    edition,
                    producer_contacts,
                )
            }
        }?;
        Ok(PreparedProducerExecutionOutcome::new(
            outcome,
            ready_backing,
        ))
    }
}
