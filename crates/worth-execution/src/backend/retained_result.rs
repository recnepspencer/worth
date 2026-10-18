//! Actual accepted native values; result custody is independent of freshness.
use std::sync::Arc;
use worth_foundational::PartitionIdentity;

use super::{
    admission::AdmittedBatch,
    port::{run_checked_batch_recording, BackendKind, BatchOutcome},
    settlement::records::SettlementRecords,
    ExecutionKernelReceipt, KernelContext, KernelFailure, KernelStop,
};
use crate::{
    ChargedBytes, ExecutionAllocationDenial, ExecutionMemoryReservation, ExecutionResourceLease,
    LeaseDenial,
};

#[derive(Debug)]
pub(crate) enum ResultStorageDenial {
    Admission(LeaseDenial),
    Allocation(ExecutionAllocationDenial),
    RequiresExecution(PartitionIdentity),
}
impl From<LeaseDenial> for ResultStorageDenial {
    fn from(value: LeaseDenial) -> Self {
        Self::Admission(value)
    }
}

/// A value accepted by native canonical settlement, with its own backing.
/// This grants neither input equality nor permission to skip an application
/// call. Only its owning runtime can qualify those prerequisites.
///
/// There is no constructor from a value, receipt, report or success flag.
///
/// ```compile_fail,E0599
/// use worth_execution::ExecutionRetainedResult;
/// let fabricated = ExecutionRetainedResult::new(73_u64);
/// ```
pub struct ExecutionRetainedResult<R>(Arc<SettledValue<R>>);

struct SettledValue<R> {
    value: R,
    identity: PartitionIdentity,
    work: u64,
    span: u64,
    receipt: Option<ExecutionKernelReceipt>,
    // Release the result and capture before their surviving backing grant.
    memory: ExecutionMemoryReservation,
}

impl<R> Clone for ExecutionRetainedResult<R> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<R> ExecutionRetainedResult<R> {
    pub(crate) fn same_backing(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn value(&self) -> &R {
        &self.0.value
    }
    pub fn identity(&self) -> PartitionIdentity {
        self.0.identity
    }
    pub fn charged_work(&self) -> u64 {
        self.0.work
    }
    pub fn charged_span(&self) -> u64 {
        self.0.span
    }

    pub fn native_reuse_available(&self) -> bool {
        self.0.receipt.is_some()
    }

    /// Compare native meter captures only. This is not value equivalence,
    /// source/currentness evidence or permission to carry either result.
    pub fn has_same_native_cost(&self, other: &Self) -> bool {
        self.0.work == other.0.work
            && self.0.span == other.0.span
            && self.0.receipt == other.0.receipt
    }

    /// None requires actual execution: native context observations or capture
    /// refusal did not qualify this result. Otherwise the fresh context owns
    /// cancellation, first-stop and tight native checkpoint-prefix behavior.
    pub fn try_carry<'value>(
        &'value self,
        context: &mut KernelContext<'_, '_>,
    ) -> Option<Result<&'value R, KernelStop>> {
        Some(
            context
                .carry_kernel(self.0.receipt.as_ref()?)
                .map(|()| &self.0.value),
        )
    }
}

impl<R> ChargedBytes for ExecutionRetainedResult<R> {
    fn additional_charged_bytes(&self) -> u64 {
        self.0.memory.bytes().saturating_add(
            self.0
                .receipt
                .as_ref()
                .map_or(0, ChargedBytes::additional_charged_bytes),
        )
    }
}

pub(super) fn result_bytes<R>(heap: u64) -> Option<u64> {
    let layout = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<SettledValue<R>>())
        .ok()?
        .0
        .pad_to_align();
    u64::try_from(layout.size()).ok()?.checked_add(heap)
}

pub(crate) fn run_retained_batch<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    inputs: Option<ExecutionMemoryReservation>,
    kernel: &F,
) -> Result<BatchOutcome<ExecutionRetainedResult<R>, E>, ResultStorageDenial>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send + Sync,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    let records = SettlementRecords::admit::<R>(batch.declaration(), lease)?;
    let outcome =
        run_checked_batch_recording(lease, batch, backend, &records, inputs, kernel, None);
    Ok(finish(batch.identities(), outcome, records, fresh))
}

enum Pending<R> {
    Fresh(R),
    Carried(ExecutionRetainedResult<R>),
}
impl<R: ChargedBytes> ChargedBytes for Pending<R> {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Fresh(value) => value.additional_charged_bytes(),
            Self::Carried(value) => value.additional_charged_bytes(),
        }
    }
}

pub(crate) fn run_retained_inputs_batch<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<crate::RetainedMapInput<T, R>>,
    backend: BackendKind,
    inputs: Option<ExecutionMemoryReservation>,
    kernel: &F,
) -> Result<BatchOutcome<ExecutionRetainedResult<R>, E>, ResultStorageDenial>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send + Sync,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    use crate::RetainedMapInput as Input;
    for (identity, input) in batch.identities().iter().zip(batch.values()) {
        if let Input::Carry(prior) = input
            && (prior.identity() != *identity || !prior.native_reuse_available())
        {
            return Err(ResultStorageDenial::RequiresExecution(*identity));
        }
    }
    let records = SettlementRecords::admit_with::<R>(batch.declaration(), lease, |index| {
        matches!(batch.values()[index], Input::Evaluate(_))
    })?;
    let outcome = run_checked_batch_recording(
        lease,
        batch,
        backend,
        &records,
        inputs,
        &|input, context| match input {
            Input::Evaluate(value) => kernel(value, context).map(Pending::Fresh),
            Input::Carry(prior) => {
                prior
                    .try_carry(context)
                    .expect("native eligibility checked before dispatch")?;
                Ok(Pending::Carried(prior.clone()))
            }
        },
        None,
    );
    Ok(finish(
        batch.identities(),
        outcome,
        records,
        |value, identity, evaluated, memory| match value {
            Pending::Fresh(value) => fresh(value, identity, evaluated, memory),
            Pending::Carried(value) => value,
        },
    ))
}

pub(crate) fn run_retained_sparse_batch<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    tree: &crate::reduction::RunTree<R>,
    metrics: &mut crate::RetainedRunMetrics,
    kernel: &F,
) -> Result<BatchOutcome<ExecutionRetainedResult<R>, E>, ResultStorageDenial>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send + Sync,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    let records = SettlementRecords::admit::<R>(batch.declaration(), lease)?;
    let mut retained = super::settlement::RetainedSettlement::new(tree, metrics);
    let outcome = run_checked_batch_recording(
        lease,
        batch,
        backend,
        &records,
        None,
        kernel,
        Some(&mut retained),
    );
    Ok(finish(batch.identities(), outcome, records, fresh))
}

fn fresh<R: ChargedBytes>(
    value: R,
    identity: PartitionIdentity,
    evaluated: super::settlement::records::EvaluatedTask,
    memory: Option<ExecutionMemoryReservation>,
) -> ExecutionRetainedResult<R> {
    let mut memory = memory.expect("fresh result admitted before evaluation");
    memory
        .resize(
            result_bytes::<R>(
                evaluated
                    .result_heap
                    .expect("accepted native result has a checked heap bound"),
            )
            .expect("validated native capacity"),
        )
        .expect("shrinking an accepted result's grant");
    ExecutionRetainedResult(Arc::new(SettledValue {
        value,
        identity,
        work: evaluated.work,
        span: evaluated.span,
        receipt: evaluated.receipt,
        memory,
    }))
}

fn finish<V, R, E>(
    identities: &[PartitionIdentity],
    outcome: BatchOutcome<V, E>,
    records: SettlementRecords,
    convert: impl Fn(
        V,
        PartitionIdentity,
        super::settlement::records::EvaluatedTask,
        Option<ExecutionMemoryReservation>,
    ) -> ExecutionRetainedResult<R>,
) -> BatchOutcome<ExecutionRetainedResult<R>, E> {
    // Settlement already rejected suffix values and retained request ownership.
    // Move, never clone, accepted results while every original grant is live.
    let values = outcome
        .values
        .into_iter()
        .zip(identities)
        .zip(records.into_slots())
        .map(|((value, identity), slot)| {
            let evaluated = slot
                .evaluated
                .into_inner()
                .expect("accepted result was evaluated");
            convert(value, *identity, evaluated, slot.memory)
        })
        .collect();
    BatchOutcome {
        values,
        stop: outcome.stop,
        prefix_boundary: outcome.prefix_boundary,
        report: outcome.report,
        stop_owner: outcome.stop_owner,
    }
}

#[cfg(test)]
mod interruption_tests;
#[cfg(test)]
mod reuse_tests;
#[cfg(test)]
mod tests;
