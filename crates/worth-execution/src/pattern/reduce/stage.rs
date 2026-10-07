use crate::{
    authority::ExecutionResourceLease,
    backend::{BackendKind, KernelContext},
    oracle::CanonicalBits,
    reduction::{ReductionPlan, ReductionTree, ScheduledReductionError},
    report::ChargedBytes,
};

use super::{ReduceComplete, ReduceScopeError};

pub(super) fn execute<R, F>(
    lease: &ExecutionResourceLease<'_>,
    backend: BackendKind,
    plan: ReductionPlan,
    values: Vec<R>,
    identity: R,
    combine: F,
    max_value_bytes: u64,
    context: &mut KernelContext<'_, '_>,
) -> Result<ReduceComplete<R, F>, ReduceScopeError>
where
    R: Send + Sync + Clone + ChargedBytes + CanonicalBits,
    F: Fn(&R, &R) -> R + Sync,
{
    let (tree, metrics) = ReductionTree::build_scheduled_checked(
        lease,
        backend,
        plan,
        values,
        identity,
        combine,
        max_value_bytes,
        context,
    )
    .map_err(|error| match error {
        ScheduledReductionError::Admission(denial) => ReduceScopeError::Admission(denial),
        ScheduledReductionError::Reduction(failure) => ReduceScopeError::Reduction(failure),
    })?;
    Ok(ReduceComplete { tree, metrics })
}
