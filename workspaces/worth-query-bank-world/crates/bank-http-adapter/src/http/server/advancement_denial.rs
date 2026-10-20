use crate::{BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next};

// Reuse the existing execution projection; this slice adds custody, not new
// HTTP outcome categories.
pub(super) fn advancement(
    denial: worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial as Denial;
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption as Interruption;
    match denial {
        Denial::Resource(cause) => super::mutation_application::execution_resource(cause),
        Denial::Interrupted(Interruption::Cancelled) => {
            BankHttpDenial::new(Kind::Cancelled, Next::Retry)
        }
        Denial::Interrupted(Interruption::DeadlineExceeded) => {
            BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
        }
        Denial::NestedOpening => BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
        Denial::NestedStopped | Denial::Panicked => {
            BankHttpDenial::new(Kind::InternalDenied, Next::None)
        }
    }
}

// Construction and admitted decoding share the native request refusal projection.
pub(super) fn request(
    cause: worth_query_host::facade::runtime::WorkCeilingDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::{
        primary_graph::WorthQueryProviderSessionControlStopKind as Control,
        runtime::{LeaseDenial, MapKernelStop, MemoryLimitLevel, WorkCeilingDenial as Request},
    };
    match cause {
        Request::Stopped(MapKernelStop::Cancelled) => {
            super::mutation_application::pending_execution_stop(Err(Control::Cancelled))
        }
        Request::Stopped(MapKernelStop::DeadlineElapsed) => {
            super::mutation_application::pending_execution_stop(Err(Control::TimedOut))
        }
        Request::Admission(LeaseDenial::MemoryExhausted(limit)) => match limit {
            worth_query_host::facade::runtime::MemoryLimitDenial::CounterOverflow { .. } => {
                operator_denial()
            }
            worth_query_host::facade::runtime::MemoryLimitDenial::Capacity { level, .. } => {
                match level {
                    MemoryLimitLevel::Process => {
                        BankHttpDenial::new(Kind::Unavailable, Next::Retry)
                    }
                    MemoryLimitLevel::Policy { .. } | MemoryLimitLevel::Declared => {
                        BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest)
                    }
                }
            }
        },
        Request::Admission(
            LeaseDenial::WorkerLimitExceedsParent
            | LeaseDenial::MemoryLimitExceedsParent
            | LeaseDenial::WorkLimitExceedsParent
            | LeaseDenial::ChargedBytesOverflow
            | LeaseDenial::UnrelatedNestedLease
            | LeaseDenial::NoActiveExecutionScope
            | LeaseDenial::EquivalenceContractUnavailable,
        )
        | Request::Stopped(
            MapKernelStop::WorkCounterOverflow
            | MapKernelStop::WorkCeiling
            | MapKernelStop::NestedStopped,
        )
        | Request::Panicked => operator_denial(),
    }
}

pub(super) fn physical_allocation(
    cause: worth_query_host::facade::runtime::ExecutionPhysicalAllocationDenialKind,
) -> BankHttpDenial {
    use worth_query_host::facade::runtime::ExecutionPhysicalAllocationDenialKind as Physical;
    match cause {
        Physical::Allocator => BankHttpDenial::new(Kind::Unavailable, Next::Retry),
        Physical::Layout
        | Physical::CapacityMismatch
        | Physical::WriteBeyondReserved
        | Physical::IncompleteSeal => operator_denial(),
    }
}

fn operator_denial() -> BankHttpDenial {
    BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
}

#[cfg(test)]
mod tests;
