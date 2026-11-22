//! Sealed test doors over this module's private members.
use super::*;

impl<Stopped> WorthQueryPartitionedComputationDenial<Stopped> {
    #[doc(hidden)]
    pub fn from_reduction_for_test(failure: ReductionRunFailure<MapKernelStop>) -> Self {
        Self::from_reduction(failure)
    }
}
