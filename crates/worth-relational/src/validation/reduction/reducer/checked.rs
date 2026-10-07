use std::{cell::RefCell, mem::size_of};

use worth_execution::{ChargedBytes, ExecutionResourceLease, MapKernelFailure, ScanOutcome};
use worth_foundational::PartitionIdentity;

use super::{reduce_inner, ReducedInvariants};
use crate::authority::commit::preparation::planning::strategy::PreparationStrategy;
use crate::execution::{PacketBudgetDenial, PacketExecutionStop, RequestWorkBudget};
use crate::validation::engine::{InvariantExecutionRequest, InvariantProofBoundarySummary};
use crate::validation::execution::InvariantWorkerEnvelope;

struct ReductionInput {
    envelopes: RefCell<Option<Vec<InvariantWorkerEnvelope>>>,
    owned_bytes: u64,
}

impl ChargedBytes for ReductionInput {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

struct ReductionOutput {
    reduced: ReducedInvariants,
    charged_bytes: u64,
}

impl ChargedBytes for ReductionOutput {
    fn additional_charged_bytes(&self) -> u64 {
        self.charged_bytes
    }
}

pub(crate) fn reduce_invariant_execution_checked(
    request: &InvariantExecutionRequest<'_>,
    strategy: PreparationStrategy,
    proof_boundary: InvariantProofBoundarySummary,
    envelopes: Vec<InvariantWorkerEnvelope>,
    lease: &ExecutionResourceLease<'_>,
    work_budget: Option<&RequestWorkBudget>,
) -> Result<ReducedInvariants, PacketExecutionStop> {
    let owned_bytes = (envelopes.capacity() as u64)
        .saturating_mul(size_of::<InvariantWorkerEnvelope>() as u64)
        .saturating_add(
            envelopes
                .iter()
                .map(InvariantWorkerEnvelope::owned_allocation_capacity_bytes)
                .fold(0_u64, u64::saturating_add),
        );
    let identity = PartitionIdentity::new(1);
    let scan = crate::execution::admit_ordered_scan(vec![(
        identity,
        ReductionInput {
            envelopes: RefCell::new(Some(envelopes)),
            owned_bytes,
        },
    )])?;
    let ceiling = owned_bytes
        .saturating_mul(12)
        .saturating_add(8192)
        .min(lease.policy().budget().charged_memory_bytes() / 3);
    let proof_boundary = RefCell::new(Some(proof_boundary));
    let outcome = crate::execution::run_with_remaining_request_work(
        lease,
        work_budget,
        |child| {
            scan.run(Some(child), (), 0, ceiling, 0, ceiling, |_, input, context| -> Result<((), ReductionOutput), MapKernelFailure<PacketBudgetDenial>> {
            context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
            let envelopes = input.envelopes.borrow_mut().take().expect("one reduction step");
            let mut claimed = 0_u64;
            let reduced = reduce_inner(request, strategy, proof_boundary.borrow_mut().take().expect("one reduction step"), envelopes, |work, bytes| -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
                context.checkpoint(work).map_err(MapKernelFailure::Stop)?;
                claimed = claimed.checked_add(bytes)
                    .filter(|total| *total <= ceiling)
                    .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
                Ok(())
            })?;
            Ok(((), ReductionOutput { reduced, charged_bytes: claimed }))
        })
        },
        ScanOutcome::report,
    );
    match outcome {
        ScanOutcome::Complete { mut prefixes, .. } => {
            Ok(prefixes.pop().expect("one reduction output").reduced)
        }
        ScanOutcome::Stopped {
            boundary, reason, ..
        } => Err(PacketExecutionStop::Execution { boundary, reason }),
    }
}
