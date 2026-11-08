//! A fresh demand may adopt a completed coupled publication without executing it again.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryObservedSource, output_lineage::RetainedInputCutoffCandidate,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitOutcome,
    WorthQuerySelectedProductOperation,
};
pub(super) struct CoupledAdoption {
    candidate: RetainedInputCutoffCandidate,
    receipt: WorthQueryApplicationCommitReceipt,
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
    {
        // Completion comes from this actual performed coupled writer, not
        // source ownership. Independent routes carry the same sealed receipt.
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
            Ok(()) => Ok(Some(Self { candidate, receipt })),
            Err(denial) if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded => {
                Ok(None)
            }
            Err(denial) => Err(denial.into()),
        }
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
        commit_receipt(
            producer,
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(self.receipt),
        )
        .map(ProducerExecutionOutcome::Committed)
        .map_err(ProducerExecutionStop::ExecutionStopped)
    }
}
