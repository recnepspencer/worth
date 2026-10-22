//! The denial's payload is charged by the type that owns its allocation.
use super::*;
use worth_execution::ChargedBytes;
impl<S: ChargedBytes> ChargedBytes for WorthQueryComputationPartitionStop<S> {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Owner(stopped) => stopped.additional_charged_bytes(),
            Self::Read(denial) => denial.additional_charged_bytes(),
            Self::Panicked
            | Self::Resource(_)
            | Self::Interrupted(_)
            | Self::NestedPatternStopped => 0,
        }
    }
}
impl<S: ChargedBytes> ChargedBytes for WorthQueryPartitionedComputationDenial<S> {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::InputNotEncodable(denial)
            | Self::ItemNotEncodable { item: _, denial }
            | Self::KeyNotEncodable { item: _, denial } => denial.retained_heap_bytes() as u64,
            Self::Owner(stopped) => stopped.additional_charged_bytes(),
            Self::Read(denial) => denial.additional_charged_bytes(),
            Self::Partition {
                partition: _,
                cause,
            } => cause.additional_charged_bytes(),
            Self::PartitionIdentityCollision { partition: _ }
            | Self::DuplicateItem { item: _ }
            | Self::Resource(_)
            | Self::Interrupted(_)
            | Self::NestedPatternStopped
            | Self::PreparationPanicked
            | Self::ReducerPanicked
            | Self::ReducedEncodingInvalid
            | Self::RetainedResultUnqualified { .. }
            | Self::ReductionInputInvalid(_) => 0,
        }
    }
}
