//! Hot completion custody for an already performed coupled producer.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationLandedCommit,
    application_query::AppliedSourcePublication, WorthQueryApplicationCommitReceipt,
};
use crate::domain_computation::primary_graph::{
    application_query, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
#[derive(Clone)]
pub struct CoupledOutputCompletion {
    source: AppliedSourcePublication,
    receipt: Arc<worth_execution::ExecutionArray<CoupledCompletionCells>>,
}
struct CoupledCompletionCells {
    receipt: OnceLock<WorthQueryApplicationLandedCommit>,
    // The lineage owner retains this completion inside the same record cell.
    // This backlink is consulted only during its canonical publication join.
    lineage: OnceLock<std::sync::Weak<OnceLock<RecordedOutput>>>,
}
impl CoupledOutputCompletion {
    pub fn same_writer(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.receipt, &other.receipt)
    }
    pub fn bind_lineage(&self, slot: &PreparedOutputLineageSlot) {
        assert!(self.receipt[0]
            .lineage
            .set(Arc::downgrade(&slot.record_cell))
            .is_ok());
    }
    pub fn settled_consumption(
        &self,
        selected: worth_relational::facade::runtime::PositionedRelationalSnapshot,
        role: &str,
        entity: worth_relational::facade::identity::EntityId,
    ) -> crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence
    {
        let cell = self.receipt[0]
            .lineage
            .get()
            .and_then(std::sync::Weak::upgrade)
            .expect("canonical publication retains the actual lineage owner");
        let record = cell
            .get()
            .expect("an earlier canonical participant recorded its authentic settlement");
        assert_eq!(
            record.correspondence.active_entity_for_role(role),
            Some(entity)
        );
        crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence::settled(
            record.computation_source, Arc::clone(&record.settlement_identity),
            record.observed_source_facts().and_then(|facts| facts.for_comparison()),
            Arc::clone(&record.consumed_outputs), record.verification_requirement(),
            record.native_output_witness_cell().map(Arc::clone), selected,
        )
    }

    pub fn prepare(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: AppliedSourcePublication,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let map = |cause| {
            match cause {
            worth_execution::ExecutionRequestArrayDenial::Request(cause) => WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ExecutionRequest(crate::domain_computation::primary_graph::application_contribution::execution_denial::advancement_scope_stop(cause)), "coupled completion custody"),
            worth_execution::ExecutionRequestArrayDenial::Allocation(_) => WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded, "coupled completion custody"),
        }
        };
        let mut receipt =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, 1).map_err(map)?;
        receipt
            .push(CoupledCompletionCells {
                receipt: OnceLock::new(),
                lineage: OnceLock::new(),
            })
            .map_err(map)?;
        Ok(Self {
            source,
            receipt: Arc::new(receipt.seal().map_err(map)?),
        })
    }
    pub fn completed_receipt(
        &self,
        settlement: &Arc<RecordedSettlementIdentity>,
    ) -> Option<WorthQueryApplicationCommitReceipt> {
        let WorthQueryApplicationLandedCommit::Live { receipt } = self.receipt[0].receipt.get()?
        else {
            return None;
        };
        receipt
            .exact_output_settlement()
            .filter(|identity| Arc::ptr_eq(identity, settlement))?;
        Some(receipt.clone())
    }
    fn receipt_for_source(
        &self,
        identity: Option<RecordedSourceIdentity>,
        runtime: application_query::WorthQueryRuntimeSourceIdentity,
        checkpoint: application_query::WorthQueryCheckpointSourceIdentity,
        settlement: &Arc<RecordedSettlementIdentity>,
    ) -> Option<WorthQueryApplicationCommitReceipt> {
        if !identity?.matches_current(runtime, checkpoint) {
            return None;
        }
        self.completed_receipt(settlement)
    }
    fn receipt_for_observed_source<Query>(
        &self,
        identity: Option<RecordedSourceIdentity>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &application_query::WorthQueryObservedSource<Query>,
        settlement: &Arc<RecordedSettlementIdentity>,
    ) -> Result<
        Option<WorthQueryApplicationCommitReceipt>,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    > {
        let Some(receipt) = self.completed_receipt(settlement) else {
            return Ok(None);
        };
        let exact = identity.is_some_and(|identity| {
            identity.matches_current(source.idempotency_identity(), source.checkpoint_identity())
        });
        if exact || self.source.matches_published_source(execution, source)? {
            Ok(Some(receipt))
        } else {
            Ok(None)
        }
    }
    pub fn source(&self) -> &AppliedSourcePublication {
        &self.source
    }
    pub fn complete(&self, receipt: WorthQueryApplicationLandedCommit) {
        assert!(
            self.receipt[0].receipt.set(receipt).is_ok(),
            "one actual managed completion for the coupled writer"
        );
    }
}
impl input_cutoff::RetainedInputCutoffCandidate {
    pub fn completed_coupled_receipt<Query>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &application_query::WorthQueryObservedSource<Query>,
    ) -> Result<
        Option<WorthQueryApplicationCommitReceipt>,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    > {
        let recorded = self.recorded();
        let Some(completion) = &recorded.coupled_completion else {
            return Ok(None);
        };
        completion.receipt_for_observed_source(
            recorded.source_identity,
            execution,
            source,
            &recorded.settlement_identity,
        )
    }
}

impl current_output::retained_candidates::WorthQueryRetainedOutputCandidate {
    pub fn completed_coupled_receipt_for_source<Query>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &application_query::WorthQueryObservedSource<Query>,
    ) -> Result<
        Option<WorthQueryApplicationCommitReceipt>,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    > {
        let Some(completion) = &self.coupled_completion else {
            return Ok(None);
        };
        completion.receipt_for_observed_source(
            self.source_identity,
            execution,
            source,
            &self.settlement_identity,
        )
    }
    pub fn matches_published_source<Query>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &application_query::WorthQueryObservedSource<Query>,
    ) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        let Some(completion) = &self.coupled_completion else {
            return Ok(false);
        };
        if completion
            .completed_receipt(&self.settlement_identity)
            .is_none()
        {
            return Ok(false);
        }
        completion
            .source
            .matches_published_source(execution, source)
    }

    pub fn completed_coupled_receipt(
        &self,
        runtime: application_query::WorthQueryRuntimeSourceIdentity,
        checkpoint: application_query::WorthQueryCheckpointSourceIdentity,
    ) -> Option<WorthQueryApplicationCommitReceipt> {
        self.coupled_completion.as_ref()?.receipt_for_source(
            self.source_identity,
            runtime,
            checkpoint,
            &self.settlement_identity,
        )
    }
}
