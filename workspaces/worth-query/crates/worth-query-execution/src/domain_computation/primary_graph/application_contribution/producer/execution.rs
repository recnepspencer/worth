use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
mod contact_observation;
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationInputScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolution;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRole;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

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
mod denial;
mod input_cutoff;
mod input_identity;
mod input_reuse;
mod installed;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) mod outcome;
mod post_authorization;
pub(in crate::domain_computation::primary_graph) mod prepared_plan;
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
    InstalledSelectedProducerExecutor<Schema> + Send + Sync
{
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
        commit_authority: WorthQueryProducerCommitAuthority,
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
        commit_authority: WorthQueryProducerCommitAuthority,
        edition: super::InstalledProducerEdition,
        producer_contacts: &mut usize,
    ) -> Result<PreparedProducerExecutionOutcome, ProducerExecutionStop> {
        let execution = phase
            .execution_request_for(&runtime.product_runtime)
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
                    commit_authority,
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
