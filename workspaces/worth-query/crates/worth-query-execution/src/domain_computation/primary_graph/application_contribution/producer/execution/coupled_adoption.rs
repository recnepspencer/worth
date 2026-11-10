//! Readmit an actual coupled completion through ordinary publication progression.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryObservedSource, output_lineage::RetainedInputCutoffCandidate,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitOutcome,
    WorthQuerySelectedProductOperation,
};
pub(super) struct CoupledAdoption {
    candidate: RetainedInputCutoffCandidate,
    receipt: WorthQueryApplicationCommitReceipt,
    posture: CompletionPosture,
}
enum CompletionPosture {
    Current,
    /// Completed historical evidence only; ordinary Ready must reobserve it.
    OwnEffectContinuation,
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
        successor_of: Option<[u8; 32]>,
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
        // Only a born-stale completion requires a genuine own-effect
        // successor. Healthy completions retain master's current readmission,
        // including a required-wave verification's successor request.
        if successor_of.is_some() && candidate.requires_own_effect_continuation() {
            return Ok(None);
        }
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
        if candidate.requires_own_effect_continuation()
            && receipt
                .committed_product_publication()
                .is_selected_at(selected.product().observation())
        {
            return Ok(Some(Self {
                candidate,
                receipt,
                posture: CompletionPosture::OwnEffectContinuation,
            }));
        }
        // Only the Current route reads the witness. A completion that keeps
        // facts without it cannot answer, so it is not adopted here and the
        // ordinary path runs the producer (public #126). The born-stale branch
        // above never reads the witness and keeps its successor chain.
        if candidate.retains_facts_without_witness() {
            return Ok(None);
        }
        match selected.require_current_output_receipts(phase, [&receipt]) {
            Ok(()) => Ok(Some(Self {
                candidate,
                receipt,
                posture: CompletionPosture::Current,
            })),
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
        match self.posture {
            CompletionPosture::Current => {
                current.require_current_output_receipts(phase, [&self.receipt])?
            }
            CompletionPosture::OwnEffectContinuation => {
                // No Current authority is issued. The normal committed Ready
                // path must observe this receipt's own-effect supersession.
                if !self
                    .receipt
                    .committed_product_publication()
                    .is_selected_at(current.product().observation())
                {
                    return Err(denial(
                        WorthQueryOutputDemandDenialKind::PublicationStale,
                        producer,
                    )
                    .into());
                }
            }
        }
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
