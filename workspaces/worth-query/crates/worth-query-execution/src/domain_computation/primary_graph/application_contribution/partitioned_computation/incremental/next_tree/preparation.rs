//! A prepared incremental run holds exactly its carried and gathered partitions.
use super::*;
impl<Key, Item, Reduced, Gathered> PreparedIncremental<Key, Item, Reduced, Gathered>
where
    Key: Send + Sync + 'static,
    Gathered: Send + Sync + worth_execution::ChargedBytes,
{
    pub(in super::super) fn new(
        remaining_work: RemainingWork,
        declared_bytes: u64,
        basis: RetainedBasis,
        prior: TypedPrior<Key, Item, Reduced>,
        validated_at: Option<worth_relational::facade::snapshots::SnapshotHandle>,
        next: NextPartitioning<Item>,
        removed: CanonicalUniqueVec<PartitionIdentity>,
    ) -> Self {
        Self {
            remaining_work,
            declared_bytes,
            partitions: BTreeMap::new(),
            next,
            removed,
            basis,
            prior,
            validated_at,
            ranges: None,
            rekeyed: Default::default(),
            transition: None,
            key_memory: None,
        }
    }
    pub(in super::super) fn transition(
        &mut self,
        transition: Arc<
            crate::domain_computation::primary_graph::invariant_projection::CallShapeTransition,
        >,
    ) {
        self.transition = Some(transition);
    }
    pub(in super::super) fn rekeyed(
        &mut self,
        calls: worth_execution::ExecutionOrderedMap<worth_execution::PartitionItemId, ()>,
        memory: QueryMemoryReservation,
    ) {
        self.rekeyed = calls;
        self.key_memory = Some(memory);
    }
    pub(in super::super) fn ranges(
        &mut self,
        ranges: Arc<
            crate::domain_computation::primary_graph::application_attempt::NativeCallCarriage,
        >,
    ) {
        self.ranges = Some(ranges);
    }

    pub(in super::super::super) const fn remaining_work(&self) -> RemainingWork {
        self.remaining_work
    }

    /// Readies one marked partition's kernel.
    pub(in super::super) fn gathered<Stopped>(
        &mut self,
        identity: PartitionIdentity,
        key: Arc<Key>,
        key_bytes: u64,
        members: Arc<[PartitionItemId]>,
        gathered: Gathered,
        gather: Option<ComputationCallCharge>,
        mut memory: GatheredMemory,
    ) -> Result<(), WorthQueryPartitionedComputationDenial<Stopped>> {
        let value = GatheredComputationPartition {
            identity,
            key: Arc::clone(&key),
            items: members,
            gathered,
        };
        memory.after_gather(&value)?;
        self.partitions.insert(
            identity,
            Recomputed {
                value,
                key,
                key_bytes,
                gather,
                memory,
            },
        );
        Ok(())
    }
}
