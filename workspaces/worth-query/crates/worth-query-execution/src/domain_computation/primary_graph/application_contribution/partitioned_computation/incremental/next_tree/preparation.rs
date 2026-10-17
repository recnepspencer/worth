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
    ) -> Self {
        Self {
            remaining_work,
            declared_bytes,
            partitions: BTreeMap::new(),
            next,
            basis,
            prior,
            validated_at,
        }
    }

    pub(in super::super::super) const fn remaining_work(&self) -> RemainingWork {
        self.remaining_work
    }

    /// Carries one partition's gathering and kernel from the retained run.
    pub(in super::super) fn carried(
        &mut self,
        identity: PartitionIdentity,
        partition: &Arc<RetainedPartition<Key>>,
    ) {
        self.partitions
            .insert(identity, Disposition::Carried(Arc::clone(partition)));
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
            Disposition::Recomputed(Recomputed {
                value,
                key,
                key_bytes,
                gather,
                memory,
            }),
        );
        Ok(())
    }
}
