//! One immutable publication carried by an original own-publication continuation.
use super::*;
use crate::domain_computation::primary_graph::{
    application_checkpoint::facts::FactPayload, WorthQueryApplicationCommittedChanges,
};

/// Descriptive original evidence only: no receipt authority, Ready cell or reader.
/// No link to another publication is retained.
pub(in crate::domain_computation::primary_graph) struct OriginalProducingPublication {
    pub(in crate::domain_computation::primary_graph) changes: WorthQueryApplicationCommittedChanges,
    pub(in crate::domain_computation::primary_graph) inputs: FactPayload,
    pub(in crate::domain_computation::primary_graph) correspondence:
        Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(in crate::domain_computation::primary_graph) conditional:
        Option<worth_runtime_bridge::facade::BridgeConditionalCausalObservation>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl OriginalProducingPublication {
    /// Called only when the actual Ready publication is selected for its own
    /// unsettled source refresh. Ready carries an earlier candidate only when
    /// its own completion proved a non-creating continuation.
    pub(in crate::domain_computation::primary_graph) fn retain(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        completion: &ReadyCompletion,
    ) -> Result<Option<Arc<Self>>, WorthQueryOutputDemandDenial> {
        if let Some(original) = completion.original_producing_publication() {
            return Ok(Some(Arc::clone(original)));
        }
        let WorthQueryAcceptedOutputAuthority::Committed(receipt) = &completion.authority else {
            return Ok(None);
        };
        let Some(inputs) = receipt.performed_input_facts() else {
            return Ok(None);
        };
        if !checked(execution, |context| {
            receipt
                .committed_changes()
                .affects_output_records(context, receipt.output_correspondence())
        })? {
            return Ok(None);
        }
        let bytes = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed publication evidence layout")
            .0
            .pad_to_align()
            .size();
        let memory = execution.reserve_retained_memory(bytes as u64).map_err(|cause| {
            crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                crate::domain_computation::primary_graph::output_reuse::RetainedFactStop::Execution(cause.into()),
                "original producing publication",
            )
        })?;
        Ok(Some(Arc::new(Self {
            changes: receipt.committed_changes().clone(),
            inputs: inputs.clone(),
            correspondence: receipt.retain_output_correspondence(),
            conditional: completion.readiness.causal_observation().cloned(),
            _memory: memory,
        })))
    }

    /// Retain at most one candidate. A mutation of any carried output resets it;
    /// the successor's own Ready may then supply its actual publication. Mixed
    /// origins are deliberately not accumulated into a field history.
    pub(in crate::domain_computation::primary_graph) fn for_successor(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: Option<Arc<Self>>,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<Arc<Self>>, WorthQueryOutputDemandDenial> {
        let Some(original) = original else {
            return Ok(None);
        };
        let preserve = checked(execution, |context| {
            let current = receipt.output_correspondence();
            if original.correspondence.native_witness_roles().len()
                != current.native_witness_roles().len()
            {
                return Ok(false);
            }
            for (old, new) in original
                .correspondence
                .native_witness_roles()
                .zip(current.native_witness_roles())
            {
                crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)?;
                if (old.0, old.2, old.3) != (new.0, new.2, new.3)
                    || new.1 == crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture::Retire {
                    return Ok(false);
                }
            }
            receipt
                .committed_changes()
                .affects_output_records(context, &original.correspondence)
                .map(|changed| !changed)
        })?;
        Ok(preserve.then_some(original))
    }
}

fn checked(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inspect: impl FnOnce(
        &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<bool, worth_execution::MapKernelStop>,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    let mut result = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        result = Some(inspect(context));
    });
    let result = match (handoff, result) {
        (Ok(_), Some(result)) => result.map_err(Into::into),
        (
            Err(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped,
            )),
            Some(Err(cause)),
        ) => Err(cause.into()),
        (Err(cause), _) => Err(cause.into()),
        (Ok(_), None) => unreachable!("successful original request runs publication inspection"),
    };
    result.map_err(|cause| {
        crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
            crate::domain_computation::primary_graph::output_reuse::RetainedFactStop::Execution(
                cause,
            ),
            "producing publication continuation",
        )
    })
}
