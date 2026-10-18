//! Native result custody preserves the original owner refusal.
use super::*;
use worth_execution::{ExecutionAllocationDenialClass, RetainedMapDenial, RetainedReduceDenial};

impl<Stopped> WorthQueryPartitionedComputationDenial<Stopped> {
    pub(in super::super) fn from_retained_run(denial: worth_execution::RetainedRunDenial) -> Self {
        use worth_execution::RetainedRunDenial as Denial;
        match denial {
            Denial::Native(cause) => Self::from_retained_map(cause),
            Denial::Map(cause) => Self::from_map_overflow(cause),
            Denial::RequiresKeyless => {
                Self::ReductionInputInvalid(ReductionInput::RetainedRunRequiresKeyless)
            }
            Denial::UnknownRemoval(partition) => {
                Self::ReductionInputInvalid(ReductionInput::RetainedRunUnknownRemoval { partition })
            }
            Denial::ChangedAndRemoved(partition) => {
                Self::ReductionInputInvalid(ReductionInput::RetainedRunChangedAndRemoved {
                    partition,
                })
            }
        }
    }

    pub(in super::super) fn from_retained_map(denial: RetainedMapDenial) -> Self {
        match denial {
            RetainedMapDenial::Request(cause) => {
                Self::from_work_ceiling(cause, Self::ReducerPanicked)
            }
            RetainedMapDenial::Admission(cause) => Self::Resource(lease_denial(cause)),
            RetainedMapDenial::Allocation(cause) => match cause.classify() {
                ExecutionAllocationDenialClass::Request(cause) => {
                    Self::from_work_ceiling(cause, Self::ReducerPanicked)
                }
                ExecutionAllocationDenialClass::Physical(kind) => {
                    Self::Resource(Resource::NativeAllocationRefused {
                        kind,
                        requested_payload_bytes: cause.requested_payload_bytes(),
                    })
                }
            },
            RetainedMapDenial::RequiresExecution(partition) => {
                Self::RetainedResultUnqualified { partition }
            }
        }
    }
    pub(in super::super) fn from_retained_reduce(
        denial: RetainedReduceDenial<PartitionRefusal<Stopped>>,
    ) -> Self {
        match denial {
            RetainedReduceDenial::Retention(cause) => Self::from_retained_map(cause),
            RetainedReduceDenial::Reduction(cause) => Self::from_reduce(cause),
        }
    }
}
