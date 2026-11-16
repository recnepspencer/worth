//! Certify one real required root before its original source is discharged.
use super::super::{WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandSettlement};
use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind as Kind;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputSourceEvidence, WorthQueryPreparedRequiredOutputSource,
};
use worth_execution::{ExecutionRequest, MapKernelContext, WorkCeilingDenial};

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn retain_fulfilled_root(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        receipt: &WorthQueryOutputSourceEvidence,
        interest: &WorthQueryOutputDemandInterest,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandDenial> {
        if !Arc::ptr_eq(&self.state, &prepared.owner.state)
            || !Arc::ptr_eq(&self.state, &interest.owner.state)
            || !interest.requires_output
        {
            return Err(refused(Kind::ForeignSource));
        }
        let memory = execution
            .reserve_retained_memory(std::mem::size_of::<WorthQueryFulfilledRequiredRoot>() as u64)
            .map_err(stopped)?;
        let mut result = None;
        let outcome = execution.run_native_with_checkpoint(|context| {
            result =
                Some(self.inspect_fulfilled_root(
                    context, prepared, receipt, interest, settlement, memory,
                ));
        });
        match (outcome, result) {
            (Ok(_), Some(result)) => result,
            (
                Err(WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::NestedStopped)),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(stopped(cause)),
            (Ok(_), None) => unreachable!("completed root fulfillment callback"),
        }
    }

    fn inspect_fulfilled_root(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        receipt: &WorthQueryOutputSourceEvidence,
        interest: &WorthQueryOutputDemandInterest,
        settlement: &WorthQueryOutputDemandSettlement,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandDenial> {
        checkpoint(context)?;
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custody = state
            .source_custody
            .get(&prepared.source_commit)
            .filter(|_| {
                super::post_commit::custody_reader::visible(&state, &prepared.source_commit)
            })
            .ok_or_else(|| refused(Kind::RetainedBasisUnavailable))?;
        if custody.retired.is_some()
            || custody.completed
            || custody.token_count == 0
            || custody.occurrence != prepared.product_occurrence
            || !custody.retired_sources.is_empty()
        {
            return Err(refused(Kind::ForeignSource));
        }
        let source = custody
            .source
            .as_ref()
            .ok_or_else(|| refused(Kind::RetainedBasisUnavailable))?;
        if !source.receipt.same_retained_output_source_as(receipt)
            || receipt.source_commit() != &prepared.source_commit
            || !receipt.is_selected_at(&source.observation)
        {
            return Err(refused(Kind::ForeignSource));
        }
        let bound = custody
            .bound_sources
            .as_deref()
            .ok_or_else(|| refused(Kind::ForeignSource))?;
        // A fixed required root binds one source; discovered tuples have their
        // own owner and are not reinterpreted by this completion.
        if bound.len() != 1
            || custody.consumed_sources.len() != 1
            || bound[0].identity != custody.consumed_sources[0]
        {
            return Err(refused(Kind::ForeignSource));
        }
        let record = state
            .records
            .get(&interest.key)
            .ok_or_else(|| refused(Kind::Closed))?;
        if record.interests == 0
            || record.required_interests == 0
            || record.product_occurrence != prepared.product_occurrence
            || record.source_scope != Some(bound[0].scope)
            || settlement.output_family_identity() != interest.key.family_identity()
            || settlement.producer_identity() != interest.key.producer_identity()
            || settlement.retained_read_ref().branch_incarnation() != prepared.product_occurrence
        {
            return Err(refused(Kind::ForeignDemand));
        }
        let mut linked = false;
        for commit in record.source_commits.iter().rev() {
            checkpoint(context)?;
            if commit == &prepared.source_commit {
                linked = true;
                break;
            }
        }
        if !linked {
            return Err(refused(Kind::ForeignSource));
        }
        let DemandState::Output(output) = &record.state else {
            return Err(refused(Kind::SchedulingDeferred));
        };
        let Some(WorthQueryOutputCheckpoint::Ready(ready)) = &output.checkpoint else {
            return Err(refused(Kind::SchedulingDeferred));
        };
        if !settlement.matches_authority(&ready.authority) {
            return Err(refused(Kind::Superseded));
        }
        checkpoint(context)?;
        Ok(WorthQueryFulfilledRequiredRoot {
            source: source.observation.clone(),
            output: Arc::clone(settlement.retained_read_ref()),
            source_intent: receipt.subject(),
            posture: settlement.posture(),
            _memory: memory,
        })
    }
}
fn refused(kind: Kind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, "")
}
fn stopped(cause: WorkCeilingDenial) -> WorthQueryOutputDemandDenial {
    refused(Kind::ExecutionRequest(
        crate::domain_computation::primary_graph::application_contribution::advancement_denial(
            cause,
        ),
    ))
}
fn checkpoint(context: &mut MapKernelContext<'_, '_>) -> Result<(), WorthQueryOutputDemandDenial> {
    (if context.has_accounting() {
        context.checkpoint(1)
    } else {
        context.poll()
    })
    .map_err(|cause| stopped(WorkCeilingDenial::Stopped(cause)))
}
