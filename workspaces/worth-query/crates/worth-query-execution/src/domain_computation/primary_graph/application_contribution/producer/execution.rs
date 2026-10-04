use std::{any::Any, sync::Arc};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, TypedMutationPreconditions,
};

use super::{
    ProducerSourceBinding as SourceBinding, ProducerSourceQuery as SourceQuery,
    ProducerSourceValue as SourceValue, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider, WorthQueryProducerCommitAuthority,
};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    RequiredOutputDemandContext, RequiredOutputExecution, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationIdempotencyBinding, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};

use super::demand::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};

mod admitted;
mod denial;
mod input_cutoff;
mod input_identity;
mod input_reuse;
mod installed;
mod outcome;
mod post_authorization;
mod readiness;
mod required_cue;
mod selected;
mod selected_public;
mod source_readmission;
use selected::InstalledSelectedProducerExecutor;
use super::demand::disclosure::{
    FreshOutputDisclosure, ValidatedOutputDisclosure, ValidatedProducerInput,
};
use admitted::execute_fresh;
pub(super) use denial::ProducerExecutionStop;
use denial::{
    denial, execution_failed, failed, identity_unavailable, query_admission_denied,
    principal_rejected, query_execution_denied, request_authority_denied,
    scope_rejected,
};
use input_identity::encode_input;
use input_reuse::{prepared_key, require_selected_program};
pub(super) use installed::TypedInstalledProducer;
use outcome::PreparedProducerExecutionOutcome;
pub(super) use outcome::ProducerExecutionOutcome;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use required_cue::RequiredCueProgress;
use outcome::{commit_receipt, completed_handler};
use source_readmission::readmit_source;

type Operation<Schema, Binding> =
    <Binding as WorthQueryApplicationProducerBinding<Schema>>::Operation;

pub(super) trait InstalledProducerExecutor<Schema>:
    InstalledSelectedProducerExecutor<Schema> + Send + Sync
{
    fn execute(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        required_output: RequiredOutputExecution,
        input: ValidatedProducerInput<'_>,
        successor_of: Option<[u8; 32]>,
        commit_authority: WorthQueryProducerCommitAuthority,
        edition: super::InstalledProducerEdition,
        limits: WorthQueryOutputDemandLimits,
        request_admission: &mut InvalidationEditAdmission,
        producer_contacts: &mut usize,
    ) -> Result<PreparedProducerExecutionOutcome, ProducerExecutionStop>;

    fn resources(&self, source: &dyn Any) -> Option<super::WorthQueryProducerDemandResources>;

    fn readiness_record(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
        admission: Option<&mut InvalidationEditAdmission>,
    ) -> Result<
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        WorthQueryOutputDemandDenial,
    >;

    fn preserved_readiness_output(&self, receipt: &WorthQueryApplicationCommitReceipt) -> bool;
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
    fn preserved_readiness_output(&self, receipt: &WorthQueryApplicationCommitReceipt) -> bool {
        receipt
            .output_correspondence()
            .posture_for_binding_role::<Binding::Operation>(Binding::OUTPUT_ROLE)
            == Ok(
                worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture::Preserve,
            )
    }

    fn readiness_record(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
        admission: Option<&mut InvalidationEditAdmission>,
    ) -> Result<
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        WorthQueryOutputDemandDenial,
    > {
        let admitted = admission.is_some();
        if let Some(admission) = admission {
            readiness::admit_readiness_record(Binding::OUTPUT_ROLE, receipt, admission)?;
        }
        let entity = receipt
            .output_correspondence()
            .entity_for_binding_role::<Binding::Operation>(Binding::OUTPUT_ROLE)
            .map_err(|error| {
                if admitted {
                    denial(WorthQueryOutputDemandDenialKind::ProducerUnavailable, "")
                } else {
                    failed(Binding::IDENTITY, error)
                }
            })?;
        Ok(
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                entity.partition_id.0,
                entity.local_slot.0,
                entity.generation.0,
            ),
        )
    }

    fn resources(&self, source: &dyn Any) -> Option<super::WorthQueryProducerDemandResources> {
        source
            .downcast_ref::<SourceValue<Schema, Binding>>()
            .map(|source| self.provider.demand_resources(source))
    }

    fn execute(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        required_output: RequiredOutputExecution,
        input: ValidatedProducerInput<'_>,
        successor_of: Option<[u8; 32]>,
        commit_authority: WorthQueryProducerCommitAuthority,
        edition: super::InstalledProducerEdition,
        limits: WorthQueryOutputDemandLimits,
        request_admission: &mut InvalidationEditAdmission,
        producer_contacts: &mut usize,
    ) -> Result<PreparedProducerExecutionOutcome, ProducerExecutionStop> {
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
                runtime,
                principal,
                request_scope,
                branch,
                required_output,
                self.provider.as_ref(),
                proof,
                successor_of,
                commit_authority,
                edition,
                limits,
                request_admission,
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
                    runtime,
                    principal,
                    request_scope,
                    branch,
                    proof.source(),
                    limits,
                    request_admission,
                    edition,
                )?;
                execute_fresh::<Schema, Binding>(
                    runtime,
                    principal,
                    request_scope,
                    branch,
                    required_output,
                    self.provider.as_ref(),
                    fresh,
                    successor_of,
                    commit_authority,
                    edition,
                    limits,
                    request_admission,
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
