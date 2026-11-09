//! A fresh demand may adopt a completed coupled publication without executing it again.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryObservedSource, output_lineage::RetainedInputCutoffCandidate,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationIdempotencyResolution as Resolution, WorthQueryApplicationUncommitted,
    WorthQuerySelectedProductOperation,
};
pub(super) struct CoupledAdoption {
    candidate: RetainedInputCutoffCandidate,
    receipt: WorthQueryApplicationCommitReceipt,
    stable: crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage,
}
impl CoupledAdoption {
    pub(super) fn select<Schema, Binding, Op, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        operation: &WorthQueryAdmittedApplicationOperation<Schema, Op, Input, Scope>,
        source: &WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
        input: &[u8; 32],
        edition: super::super::InstalledProducerEdition,
    ) -> Result<Option<Self>, ProducerExecutionStop>
    where
        Schema: ApplicationSchema + 'static,
        Binding: WorthQueryApplicationProducerBinding<Schema>,
        Input: Clone + Send + Sync + 'static,
    {
        if Binding::OWNERSHIP
            != worth_foundational::facade::ProducerPublicationOwnership::SourceScopeCascade
        {
            return Ok(None);
        }
        let handle = &runtime.runtime.primary_provider.graph;
        let candidate = handle
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .prior_input_cutoff_candidate::<Operation<Schema, Binding>>(
                operation.operation_scope_binding(),
                selected.product().observation(),
                *source.parameter_binding_identity.bytes(),
                execution,
            )
            .map_err(|cause| source_readmission::ready_resource_denial(Binding::IDENTITY, cause))?;
        let Some(candidate) = candidate else {
            return Ok(None);
        };
        let Some(receipt) = candidate
            .completed_coupled_receipt(source.idempotency_identity(), source.checkpoint_identity())
        else {
            return Ok(None);
        };
        let binding = receipt.idempotency_binding();
        if binding.intent_identity() != input
            || binding.producing_implementation()
                != Some(
                    worth_relational::facade::mvcc::RequiredOutputProducingImplementation::new(
                        edition.build_identity(),
                        edition.digest(),
                    ),
                )
        {
            return Ok(None);
        }
        match selected.require_current_output_receipts(phase, [&receipt]) {
            Ok(()) => {}
            Err(denial) if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded => {
                return Ok(None);
            }
            Err(denial) => return Err(denial.into()),
        }
        let resolution = runtime
            .resolve_admitted_application_idempotency_with_request(execution, operation, binding)
            .map_err(|stop| {
                denial(
                    WorthQueryOutputDemandDenialKind::PublicationReceipt(stop.kind()),
                    Binding::IDENTITY,
                )
            })?
            .into_resolution();
        let original = match resolution {
            Resolution::RecoveredCommitted(original) => original,
            Resolution::ReleasedOriginal(_) => return Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationReceipt(
                    crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenialKind::OriginalEffectReleased,
                ), Binding::IDENTITY,
            ).into()),
            Resolution::ProductUnpublished(unpublished) => return Err(denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable, Binding::IDENTITY,
            ).with_uncommitted(WorthQueryApplicationUncommitted::ProductUnpublished(unpublished)).into()),
            Resolution::ProductPreparationPending(pending) => return Err(denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable, Binding::IDENTITY,
            ).with_uncommitted(WorthQueryApplicationUncommitted::ProductPreparationPending(pending)).into()),
            Resolution::Unseen | Resolution::IntentDrift => return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource, Binding::IDENTITY,
            ).into()),
        };
        let stable = handle
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .resolve_original_publication::<Operation<Schema, Binding>>(
                execution,
                operation.operation_scope_binding(),
                selected.product().observation(),
                binding,
                &original,
            )
            .map_err(|cause| source_readmission::ready_resource_denial(Binding::IDENTITY, cause))?
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    Binding::IDENTITY,
                )
                .with_recovered_publication(original.clone())
            })?;
        if !std::sync::Arc::ptr_eq(stable.exact_settlement(), candidate.settlement_identity()) {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationStale,
                Binding::IDENTITY,
            )
            .with_recovered_publication(original)
            .into());
        }
        Ok(Some(Self {
            candidate,
            receipt,
            stable,
        }))
    }
    pub(super) fn publish<Schema: ApplicationSchema>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        required: RequiredOutputDemandContext,
        producer: &'static str,
    ) -> Result<ProducerExecutionOutcome, ProducerExecutionStop> {
        let lane = runtime
            .runtime
            .primary_provider
            .application_branch_commit_lane(selected.product().observation());
        let _coordination = lane.enter();
        let current = runtime
            .on_branch(selected.product().product_branch())
            .select(phase)
            .map_err(|cause| failed(producer, cause))?;
        if current.product().observation() != selected.product().observation() {
            return Err(
                denial(WorthQueryOutputDemandDenialKind::PublicationStale, producer).into(),
            );
        }
        current.require_current_output_receipts(phase, [&self.receipt])?;
        let owner = &runtime
            .runtime
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner;
        let mut claims =
            required.prepare_prerequisites(self.candidate.consumed_outputs().iter(), owner)?;
        claims.reserve_identity(self.candidate.settlement_identity())?;
        claims
            .publish(std::sync::Arc::clone(self.candidate.settlement_identity()))
            .retire(None, owner);
        Ok(ProducerExecutionOutcome::Stable(self.stable))
    }
}
