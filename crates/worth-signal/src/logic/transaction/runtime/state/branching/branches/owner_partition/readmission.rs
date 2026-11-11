//! Consume cold source seeds without synchronizing or extending their node arenas.
use super::*;
use crate::durability::{SignalOwnerReadmissionDenial as Denial, SignalRecordedSourcePoint};

pub(crate) struct SignalReadmittedCellSeed<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) handle: SignalBranchHandle,
    pub(crate) state: Box<BranchState<D, I, T>>,
    pub(crate) point: SignalRecordedSourcePoint,
    pub(crate) restored: bool,
}
impl<D, I, T> SignalOwnerPartition<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn from_readmitted(
        runtime_instance_id: u64,
        seeds: Vec<SignalReadmittedCellSeed<D, I, T>>,
        snapshots: BTreeMap<(SignalBranchId, SignalSnapshotId), SnapshotBranchState<D, I, T>>,
        next_branch_id: u64,
        next_snapshot_id: u64,
        durable_admission: crate::durability::SignalDurableOwnerAdmission,
    ) -> Result<Self, Denial> {
        if seeds.len() > crate::branch::owner_services::DEFAULT_MAXIMUM_LIVE_SIGNAL_BRANCHES
            || snapshots.len()
                > super::super::catalog::DEFAULT_MAXIMUM_STORED_SIGNAL_BRANCH_SNAPSHOTS
        {
            return Err(Denial::Capacity {
                branches: seeds.len(),
                snapshots: snapshots.len(),
            });
        }
        let selected = seeds.first().ok_or(Denial::NoBranches)?.handle.id;
        let mut children = BTreeMap::<SignalBranchId, BTreeSet<SignalBranchId>>::new();
        let mut cells = Vec::with_capacity(seeds.len());
        let mut coordinates = BTreeMap::new();
        let mut names = BTreeSet::new();
        for seed in seeds {
            if !names.insert(seed.handle.name.clone()) {
                return Err(Denial::DuplicateBranchName(seed.handle.name));
            }
            if let Some(parent) = seed.handle.parent_branch_id {
                children.entry(parent).or_default().insert(seed.handle.id);
            }
            let generation = seed.point.descriptor().generation().ordinal();
            let restored = seed
                .restored
                .then_some(seed.handle.head_snapshot_id)
                .flatten();
            coordinates.insert(seed.handle.id, seed.point);
            cells.push((seed.handle, *seed.state, generation, restored));
        }
        let mut partition = Self::new(
            SignalOwnerMetadataState {
                snapshots,
                maximum_stored_snapshots:
                    super::super::catalog::DEFAULT_MAXIMUM_STORED_SIGNAL_BRANCH_SNAPSHOTS,
                children_by_parent: children,
                retirement_reservations: BTreeSet::new(),
                retirement_receipts: BTreeMap::new(),
                active_merge_participants: BTreeSet::new(),
                maximum_retirement_receipts: MAXIMUM_RETAINED_SIGNAL_BRANCH_RETIREMENT_RECEIPTS,
                reserved_retirement_receipt_count: 0,
                next_snapshot_id,
            },
            next_branch_id,
            SignalBranchRetentionRegistry::new(runtime_instance_id),
            selected,
            cells,
        );
        partition.readmitted_coordinates = coordinates;
        partition.durable_admission = Some(durable_admission);
        Ok(partition)
    }
    pub(crate) fn take_durable_owner_admission(
        &mut self,
    ) -> Option<crate::durability::SignalDurableOwnerAdmission> {
        self.durable_admission.take()
    }
    pub(crate) fn take_readmitted_coordinates(
        &mut self,
    ) -> BTreeMap<SignalBranchId, SignalRecordedSourcePoint> {
        std::mem::take(&mut self.readmitted_coordinates)
    }
}
