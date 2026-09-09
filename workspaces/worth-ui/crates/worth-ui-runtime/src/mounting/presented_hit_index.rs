use super::spatial_index::UiMountedSpatialTree;
use super::UiPresentedHitTestRow;
use crate::mounting::UiHitTestSpatialWork;
use crate::runtime::motion::UiMotionTargetIdentity;
use crate::runtime::persistent_index::{
    UiPersistentIndexMutationWork, UiPersistentOrdMap, UiPersistentOrdSet,
};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSurfaceBindingGeneration};

mod changes;
mod query;
pub(crate) use changes::UiPresentedHitChanges;
#[cfg(test)]
mod changes_tests;
#[cfg(test)]
mod tests;
pub(in crate::mounting) use query::UiPresentedHitQuery;
pub(crate) use query::UiPresentedHitQueryDenial;

type PartitionKey = (UiSurfaceBindingGeneration, u8);

/// Reconstructible acceleration over Portal-projected baselines and their
/// committed Motion projection. It issues neither receipts nor interaction truth.
#[derive(Clone, Default)]
pub(in crate::mounting) struct UiPresentedHitIndex {
    rows: UiPersistentOrdMap<UiMountedInstanceIdentity, Record>,
    partitions: UiPersistentOrdMap<PartitionKey, Partition>,
    portal_targets:
        UiPersistentOrdMap<UiMotionTargetIdentity, UiPersistentOrdSet<UiMountedInstanceIdentity>>,
    target_member_bytes: usize,
}

#[derive(Clone, Copy, PartialEq)]
struct Record {
    base: UiPresentedHitTestRow,
    effective: Option<UiPresentedHitTestRow>,
}

// Completed boxes contain no NaN components.
impl Eq for Record {}

#[derive(Clone, Default)]
struct Partition {
    tree: UiMountedSpatialTree,
    rows: usize,
    visible_rows: usize,
}

impl UiPresentedHitIndex {
    pub(in crate::mounting) fn for_instance(
        &self,
        binding: UiSurfaceBindingGeneration,
        instance: UiMountedInstanceIdentity,
    ) -> (Option<UiPresentedHitTestRow>, usize) {
        let (record, probes) = self.rows.get_with_probes(&instance);
        (
            record
                .and_then(|record| record.effective)
                .filter(|row| row.mounted().binding() == binding),
            probes,
        )
    }

    pub(in crate::mounting) fn replace_base(
        &mut self,
        instance: UiMountedInstanceIdentity,
        row: Option<UiPresentedHitTestRow>,
    ) -> UiHitTestSpatialWork {
        let (old, probes) = self.rows.get_with_probes(&instance);
        let old = old.copied();
        let mut work = UiHitTestSpatialWork {
            map_key_probes: probes,
            ..Default::default()
        };
        if old.map(|old| old.base) == row {
            return work;
        }
        let same_partition = old.zip(row).is_some_and(|(old, new)| {
            old.base.mounted().binding() == new.mounted().binding()
                && old.base.bounds().coordinate_space() == new.bounds().coordinate_space()
        });
        if let Some(old) = old {
            if same_partition {
                self.update_partition(old.base, old.effective, row, 0, &mut work);
            } else {
                self.update_partition(old.base, old.effective, None, -1, &mut work);
            }
            if old.base.portal_motion_target() != row.and_then(|row| row.portal_motion_target()) {
                self.update_target(old.base, false, &mut work);
            }
        }
        if let Some(base) = row {
            assert_eq!(base.mounted_instance(), instance);
            if !same_partition {
                self.update_partition(base, None, Some(base), 1, &mut work);
            }
            if old.and_then(|old| old.base.portal_motion_target()) != base.portal_motion_target() {
                self.update_target(base, true, &mut work);
            }
            record_map(
                &mut work,
                self.rows.insert_with_work(
                    instance,
                    Record {
                        base,
                        effective: Some(base),
                    },
                ),
            );
        } else if old.is_some() {
            record_map(&mut work, self.rows.remove_with_work(&instance).1);
        }
        work
    }

    pub(in crate::mounting) fn apply_motion(
        &mut self,
        sampler: &super::presentation::motion_sampling::UiMountedMotionSampler,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        targets: &[UiMotionTargetIdentity],
    ) -> UiHitTestSpatialWork {
        let mut work = UiHitTestSpatialWork::default();
        let mut selected = std::collections::BTreeSet::new();
        for target in targets {
            let (members, probes) = self.portal_targets.get_with_probes(target);
            work.map_key_probes += probes;
            if let Some(members) = members {
                for instance in members.iter() {
                    work.motion_members_visited += 1;
                    selected.insert(*instance);
                }
            }
            let (row, probes) = self.rows.get_with_probes(&target.mounted_instance());
            work.map_key_probes += probes;
            if row.is_some_and(|row| {
                row.base.portal_motion_target().is_none() && !row.base.owns_presented_portal()
            }) {
                selected.insert(target.mounted_instance());
            }
        }
        for instance in selected {
            let (record, probes) = self.rows.get_with_probes(&instance);
            work.map_key_probes += probes;
            let record = *record.expect("selected Motion member remains indexed");
            if record.base.mounted().binding() != presentation.binding() {
                continue;
            }
            work.motion_rows_projected += 1;
            let (effective, tracks) = record.base.with_current_motion_work(sampler, presentation);
            work.motion_tracks_considered += tracks;
            if effective == record.effective {
                continue;
            }
            self.update_partition(record.base, record.effective, effective, 0, &mut work);
            record_map(
                &mut work,
                self.rows.insert_with_work(
                    instance,
                    Record {
                        effective,
                        ..record
                    },
                ),
            );
        }
        work
    }

    fn update_partition(
        &mut self,
        base: UiPresentedHitTestRow,
        previous: Option<UiPresentedHitTestRow>,
        next: Option<UiPresentedHitTestRow>,
        row_delta: isize,
        work: &mut UiHitTestSpatialWork,
    ) {
        let key = (
            base.mounted().binding(),
            base.bounds().coordinate_space() as u8,
        );
        let (partition, probes) = self.partitions.get_with_probes(&key);
        work.map_key_probes += probes;
        let mut partition = partition.cloned().unwrap_or_default();
        if row_delta == 0
            && previous.is_some() == next.is_some()
            && previous.map(region) == next.map(region)
        {
            return;
        }
        partition.rows = partition
            .rows
            .checked_add_signed(row_delta)
            .expect("partition membership is canonical");
        partition.visible_rows =
            partition.visible_rows - usize::from(previous.is_some()) + usize::from(next.is_some());
        work.merge(
            partition
                .tree
                .replace(
                    base.mounted_instance(),
                    previous.map(region),
                    next.map(region),
                )
                .expect("completed presented geometry has finite ordered bounds"),
        );
        let mutation = if partition.rows == 0 {
            self.partitions.remove_with_work(&key).1
        } else {
            self.partitions.insert_with_work(key, partition)
        };
        record_map(work, mutation);
    }

    fn update_target(
        &mut self,
        row: UiPresentedHitTestRow,
        insert: bool,
        work: &mut UiHitTestSpatialWork,
    ) {
        let Some(target) = row.portal_motion_target() else {
            return;
        };
        let (members, probes) = self.portal_targets.get_with_probes(&target);
        work.map_key_probes += probes;
        let mut members = members.cloned().unwrap_or_default();
        let before = members
            .retained_structural_bytes()
            .expect("member bytes fit address space");
        let mutation = if insert {
            members.insert_with_work(row.mounted_instance()).1
        } else {
            members.remove_with_work(&row.mounted_instance()).1
        };
        record_map(work, mutation);
        self.target_member_bytes = self
            .target_member_bytes
            .checked_sub(before)
            .and_then(|bytes| bytes.checked_add(members.retained_structural_bytes()?))
            .expect("member bytes fit address space");
        let mutation = if members.is_empty() {
            self.portal_targets.remove_with_work(&target).1
        } else {
            self.portal_targets.insert_with_work(target, members)
        };
        record_map(work, mutation);
    }

    /// Reserve one spatial leaf per baseline row, including currently invisible
    /// rows. Accepted Motion changes visibility without growing this reservation.
    pub(in crate::mounting) fn retained_structural_bytes(&self) -> Option<usize> {
        self.rows
            .retained_structural_bytes()?
            .checked_add(self.partitions.retained_structural_bytes()?)?
            .checked_add(self.portal_targets.retained_structural_bytes()?)?
            .checked_add(self.target_member_bytes)?
            .checked_add(UiMountedSpatialTree::structural_bytes_for_nodes(
                self.rows.len(),
            )?)
    }
}

fn record_map(work: &mut UiHitTestSpatialWork, mutation: UiPersistentIndexMutationWork) {
    work.map_key_probes += mutation.key_probes();
    work.map_node_copies += mutation.node_copies();
}

fn region(row: UiPresentedHitTestRow) -> [f64; 4] {
    let bounds = row.bounds();
    let clip = row.clip_bounds();
    let x = bounds.x().max(clip.x());
    let y = bounds.y().max(clip.y());
    let right = f64::from(bounds.x() + bounds.width())
        .min(f64::from(clip.x() + clip.width()))
        .max(f64::from(x));
    let bottom = f64::from(bounds.y() + bounds.height())
        .min(f64::from(clip.y() + clip.height()))
        .max(f64::from(y));
    [f64::from(x), f64::from(y), right, bottom]
}
