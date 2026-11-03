//! Sealed test doors over this module's private members.
use super::*;

impl ComputationFactReaders {
    #[doc(hidden)]
    pub fn partitions_for_test(&self) -> &[PartitionIdentity] {
        Self::partitions(self)
    }
}
