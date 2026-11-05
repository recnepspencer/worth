//! Physical scratch faults are descriptive; the original caller owns refusal.
use worth_execution::ExecutionMemoryReservation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionContentStorageFault {
    CapacityOverflow,
    AllocationFailed,
}

impl ProjectionContentStorageFault {
    pub(in crate::domain_computation::primary_graph) fn into_native(
        self,
    ) -> worth_relational::facade::transactions::RelationalExecutionDenialCause {
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
        match self {
            Self::CapacityOverflow => Native::ChargedBytesOverflow,
            Self::AllocationFailed => Native::AllocationFailed,
        }
    }
}
