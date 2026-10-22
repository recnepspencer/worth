//! Preserve a reducer's typed cause while its scope transports the full metrics.
use super::{KernelContext, KernelFailure, ReduceScopeError, ReductionRunStop};
use crate::backend::{NestedComputationStop as Cause, NestedStopOwner as Owner};
use crate::reduction::ReductionDenial;

impl ReduceScopeError {
    pub(super) fn record_owner(&self, context: &mut KernelContext<'_, '_>) {
        let owner = match self {
            Self::Admission(_) => Owner::Request,
            Self::Reduction(failure) => reduction_owner(context, &failure.reason),
        };
        context.record_nested_stop_owner(owner);
    }
}

pub(super) fn record_reduction(
    context: &mut KernelContext<'_, '_>,
    reason: &ReductionRunStop<crate::backend::KernelStop>,
) {
    let owner = reduction_owner(context, reason);
    context.record_nested_stop_owner(owner);
}
fn reduction_owner(
    context: &KernelContext<'_, '_>,
    reason: &ReductionRunStop<crate::backend::KernelStop>,
) -> Owner {
    match reason {
        ReductionRunStop::Hook(stop) => context.failure_owner(&KernelFailure::<()>::Stop(*stop)),
        ReductionRunStop::Panic => Owner::Computation(Cause::Panic),
        ReductionRunStop::ResultCapacityExceeded => {
            Owner::Computation(Cause::ResultCapacityExceeded)
        }
        ReductionRunStop::WorkCounterOverflow => Owner::Request,
        ReductionRunStop::Denial(denial) => match denial {
            ReductionDenial::Admission(_) => Owner::Request,
            ReductionDenial::Allocation(kind) => Owner::Computation(Cause::Allocation(*kind)),
            ReductionDenial::WorkCounterOverflow => Owner::Request,
            ReductionDenial::AccountingUnavailable => Owner::Request,
            ReductionDenial::ReducerPanic => Owner::Computation(Cause::Panic),
            ReductionDenial::ResultCapacityExceeded => {
                Owner::Computation(Cause::ResultCapacityExceeded)
            }
            ReductionDenial::IdentitiesNotCanonical
            | ReductionDenial::ValueCountMismatch
            | ReductionDenial::CoverageMismatch
            | ReductionDenial::UnknownIdentity(_)
            | ReductionDenial::IdentityAlreadyPresent(_)
            | ReductionDenial::InvalidCanonicalEncoding => Owner::Computation(Cause::DomainFailure),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_reduction_allocation_keeps_its_original_nested_kind() {
        let kind = crate::ExecutionPhysicalAllocationDenialKind::Allocator;
        let outer = crate::backend::run_scope(None, 0, 0, |context| {
            record_reduction(
                context,
                &ReductionRunStop::Denial(ReductionDenial::Allocation(kind)),
            );
            assert_eq!(
                context.take_nested_computation_stop(),
                Some(Cause::Allocation(kind))
            );
            Ok::<(), KernelFailure<()>>(())
        });
        assert!(outer.result.is_ok());
    }
}
