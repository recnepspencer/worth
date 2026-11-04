//! Sealed test doors over this module's private members.
use super::*;

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &RetainedComputation {
        &self.state
    }
}

impl RetainedComputation {
    #[doc(hidden)]
    pub fn facts_for_test(&self) -> &SealedComputationFacts {
        &self.facts
    }
}

impl RetainedComputation {
    #[doc(hidden)]
    pub fn facts_mut_for_test(&mut self) -> &mut SealedComputationFacts {
        &mut self.facts
    }
}

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn state_into_for_test(self) -> RetainedComputation {
        self.state
    }
}

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn attribution_memory_for_test(&self) -> &Option<QueryMemoryReservation> {
        &self.attribution_memory
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn call_sequence_for_test(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::invariant_projection::ComputationCallSequence>
    {
        &self.call_sequence
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn digests_for_test(&self) -> &ItemDigests {
        &self.digests
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn item_keys_for_test(
        &self,
    ) -> &Arc<worth_execution::ExecutionOrderedMap<PartitionItemId, RetainedCall>> {
        &self.item_keys
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn items_for_test(&self) -> &Items<Item> {
        &self.items
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn membership_for_test(&self) -> &RetainedCall {
        &self.membership
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn partitions_for_test(&self) -> &super::super::partition_metadata::PartitionMetadata<Key> {
        &self.partitions
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn routing_for_test(&self) -> &Arc<ComputationPartitionRouting> {
        &self.routing
    }
}

impl RetainedComputation {
    #[doc(hidden)]
    pub fn typed_for_test(&self) -> &Arc<dyn Any + Send + Sync> {
        &self.typed
    }
}

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn cloned_from_mut_for_test(&mut self) -> &mut Option<Arc<RetainedComputation>> {
        &mut self.cloned_from
    }
}

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn state_mut_for_test(&mut self) -> &mut RetainedComputation {
        &mut self.state
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn tree_mut_for_test(
        &mut self,
    ) -> &mut NativeReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced> {
        &mut self.tree
    }
}

impl RetainedComputation {
    #[doc(hidden)]
    pub fn basis_for_test(&self) -> &RetainedBasis {
        &self.basis
    }
}

impl RetainedComputation {
    #[doc(hidden)]
    pub fn basis_mut_for_test(&mut self) -> &mut RetainedBasis {
        &mut self.basis
    }
}

impl RetainedComputation {
    #[doc(hidden)]
    pub fn typed_mut_for_test(&mut self) -> &mut Arc<dyn Any + Send + Sync> {
        &mut self.typed
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn native_run_for_test(&self) -> &worth_execution::ExecutionRetainedRun<Reduced> {
        &self.native_run
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn tree_for_test(
        &self,
    ) -> &NativeReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced> {
        &self.tree
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn digests_into_for_test(self) -> ItemDigests {
        self.digests
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn item_keys_into_for_test(
        self,
    ) -> Arc<worth_execution::ExecutionOrderedMap<PartitionItemId, RetainedCall>> {
        self.item_keys
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn items_into_for_test(self) -> Items<Item> {
        self.items
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn routing_into_for_test(self) -> Arc<ComputationPartitionRouting> {
        self.routing
    }
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced> {
    #[doc(hidden)]
    pub fn routing_mut_for_test(&mut self) -> &mut Arc<ComputationPartitionRouting> {
        &mut self.routing
    }
}

impl RetainedCall {
    #[doc(hidden)]
    pub fn charge_for_test(&self) -> &ComputationCallCharge {
        &self.charge
    }
}

impl RetainedCall {
    #[doc(hidden)]
    pub fn declared_units_for_test(&self) -> &Option<u64> {
        &self.declared_units
    }
}

impl<Key> RetainedPartition<Key> {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &Arc<Key> {
        &self.key
    }
}

impl<Key> RetainedPartition<Key> {
    #[doc(hidden)]
    pub fn gather_for_test(&self) -> &ComputationCallCharge {
        &self.gather
    }
}
