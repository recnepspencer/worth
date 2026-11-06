//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationFactKey;

impl ComputationFactReaders {
    #[doc(hidden)]
    pub fn partitions_for_test(&self) -> &[PartitionIdentity] {
        Self::partitions(self)
    }
}

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn index_for_test(&self) -> &std::sync::Arc<index::ComputationFactIndex> {
        &self.index
    }
}

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn layout_for_test(&self) -> &layout::SealedReadLayout {
        &self.layout
    }
}

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn metadata_for_test(
        &self,
    ) -> &std::sync::Arc<
        worth_execution::ExecutionOrderedMap<WorthQueryApplicationFactKey, SealedComputationFact>,
    > {
        &self.metadata
    }
}

impl ComputationFactReaders {
    #[doc(hidden)]
    pub fn read_by_for_test(
        membership: bool,
        item_keys: impl IntoIterator<Item = u64>,
        partitions: impl IntoIterator<Item = PartitionIdentity>,
    ) -> Self {
        Self::read_by(membership, item_keys, partitions)
    }
}

impl SealedComputationFact {
    #[doc(hidden)]
    pub fn ordinal_for_test(&self) -> &usize {
        &self.ordinal
    }
}

impl SealedComputationFact {
    #[doc(hidden)]
    pub fn readers_for_test(&self) -> &ComputationFactReaders {
        &self.readers
    }
}
