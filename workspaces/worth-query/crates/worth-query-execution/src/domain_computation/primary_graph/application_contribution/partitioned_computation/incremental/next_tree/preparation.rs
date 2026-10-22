//! A prepared incremental run holds exactly its carried and gathered partitions.
use super::*;
impl<Key, Item, Reduced, Gathered> PreparedIncremental<Key, Item, Reduced, Gathered>
where
    Key: Send + Sync + 'static,
    Gathered: Send + Sync,
{
    pub(in super::super) fn new(
        preparation_work: PreparationWork,
        basis: RetainedBasis,
        prior: TypedPrior<Key, Item, Reduced>,
        validated_at: Option<worth_relational::facade::snapshots::SnapshotHandle>,
        next: NextPartitioning<Item>,
        removed: CanonicalUniqueVec<PartitionIdentity>,
    ) -> Self {
        Self {
            preparation_work,
            compute_ceiling: None,
            partitions: BTreeMap::new(),
            next,
            removed,
            basis,
            prior,
            validated_at,
            ranges: None,
            rekeyed: Default::default(),
            transition: None,
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
    ) {
        self.rekeyed = calls;
    }
    pub(in super::super) fn ranges(
        &mut self,
        ranges: Arc<
            crate::domain_computation::primary_graph::application_attempt::NativeCallCarriage,
        >,
    ) {
        self.ranges = Some(ranges);
    }

    pub(in super::super::super) const fn preparation_work(&self) -> PreparationWork {
        self.preparation_work
    }

    /// Readies one marked partition's kernel.
    pub(in super::super) fn gathered<Stopped>(
        &mut self,
        identity: PartitionIdentity,
        key: Arc<Key>,
        members: Arc<[PartitionItemId]>,
        gathered: Gathered,
        gather: Option<ComputationCallCharge>,
    ) -> Result<(), WorthQueryPartitionedComputationDenial<Stopped>> {
        let value = GatheredComputationPartition {
            identity,
            key: Arc::clone(&key),
            items: members,
            gathered,
        };
        self.partitions
            .insert(identity, Recomputed { value, key, gather });
        Ok(())
    }
}
