//! One exclusive writer preserves current target metadata and old editions.
mod resident_entry;
pub(crate) use resident_entry::ResidentPartitionEntry;
use resident_entry::ResidentTargetEditor;
use std::collections::BTreeMap;
use std::sync::RwLockWriteGuard;

use crate::identity::data::PartitionId;
use crate::runtime::state::subsystems::RuntimeInstrumentation;
use crate::storage::overlay::{CurrentPartitionAccess, PartitionState};
#[cfg(test)]
use crate::storage::residency::{RelationalRegionRetention, ResidentRegionLease};
use crate::storage::residency::{ResidentHistoryReadmission, ResidentRegionEdit};

use super::edition_copy_lane::{PartitionCopyTally, PartitionEditionCopyLane};
use super::{CurrentMirror, MirrorRegion, PartitionMap, ResidentMirrorTarget};

pub(crate) struct PartitionEditionWriter<'subsystem> {
    mirror: RwLockWriteGuard<'subsystem, CurrentMirror>,
    instrumentation: &'subsystem RuntimeInstrumentation,
    lane: PartitionEditionCopyLane,
    copies: PartitionCopyTally,
    edits: BTreeMap<PartitionId, TargetEdit>,
}

struct TargetEdit {
    target: ResidentMirrorTarget,
    edit: ResidentRegionEdit,
}

/// Exclusive, load-free access to a mirror whose entire edition is Full.
pub(crate) struct ResidentPartitionEditionWriter<'subsystem> {
    writer: PartitionEditionWriter<'subsystem>,
    targets: Vec<ResidentMirrorTarget>,
}

impl ResidentPartitionEditionWriter<'_> {
    pub(crate) fn visit_partitions(
        &mut self,
        mut visit: impl FnMut(
            ResidentPartitionEntry<'_>,
        ) -> Result<(), crate::execution::RelationalExecutionDenialCause>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for target in &self.targets {
            visit(ResidentPartitionEntry {
                target,
                writer: &mut self.writer,
            })?;
        }
        Ok(())
    }
}

impl<'subsystem> PartitionEditionWriter<'subsystem> {
    pub(super) fn new(
        mirror: RwLockWriteGuard<'subsystem, CurrentMirror>,
        instrumentation: &'subsystem RuntimeInstrumentation,
        lane: PartitionEditionCopyLane,
    ) -> Self {
        Self {
            mirror,
            instrumentation,
            lane,
            copies: PartitionCopyTally::default(),
            edits: BTreeMap::new(),
        }
    }

    pub(super) fn accept_resident(
        mut self,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<ResidentPartitionEditionWriter<'subsystem>, Self> {
        let mut targets = Vec::new();
        for id in self.mirror.edition.partition_ids() {
            if !matches!(
                crate::storage::overlay::PartitionAccess::probe_full_partition(
                    &self.mirror.edition.with_symbol_admission(symbols),
                    id
                ),
                crate::storage::residency::PartitionProbe::Resident(_)
            ) {
                return Err(self);
            }
            match self.mirror.admit_target(id) {
                Ok(Some(target)) => targets.push(target),
                _ => return Err(self),
            }
        }
        Ok(ResidentPartitionEditionWriter {
            writer: self,
            targets,
        })
    }

    fn map_mut(&mut self) -> &mut PartitionMap {
        let (map, copied) = self.mirror.edition.map_mut();
        if copied {
            self.copies.record_spine_copy();
        }
        map
    }

    fn begin_target_edit(&mut self, target: &ResidentMirrorTarget) -> bool {
        let id = target.partition_id();
        if let Some(entry) = self.edits.get_mut(&id) {
            return entry.edit.full_partition_mut().is_some();
        }
        if !target.is_full() {
            return false;
        }
        debug_assert!(matches!(self.mirror.regions.get(&id),
            Some(MirrorRegion::Resident(current)) if current.same_target(target)));
        // The cached edition is an owner projection, not another observer.
        // An external edition retains its old map and payload independently.
        drop(self.map_mut().remove(&id));
        let Some((edit, copied)) = target.begin_full_edit() else {
            return false;
        };
        if copied {
            self.copies.record_partition_copy(
                edit.partition().entity_arena.slot_count(),
                edit.partition().relation_arena.slot_count(),
            );
        }
        self.edits.insert(
            id,
            TargetEdit {
                target: target.clone(),
                edit,
            },
        );
        true
    }

    fn begin_current_target_edit(
        &mut self,
        target: &ResidentMirrorTarget,
        readmission: ResidentHistoryReadmission,
    ) {
        let id = target.partition_id();
        if self.edits.contains_key(&id) {
            return;
        }
        debug_assert!(
            matches!(self.mirror.regions.get(&id), Some(MirrorRegion::Resident(current)) if current.same_target(target))
        );
        drop(self.map_mut().remove(&id));
        let (edit, copied) = target.begin_current_edit(readmission);
        if copied {
            self.copies.record_partition_copy(
                edit.partition().entity_arena.slot_count(),
                edit.partition().relation_arena.slot_count(),
            );
        }
        self.edits.insert(
            id,
            TargetEdit {
                target: target.clone(),
                edit,
            },
        );
    }

    pub(crate) fn transform_target(
        &mut self,
        target: &ResidentMirrorTarget,
        readmission: ResidentHistoryReadmission,
        transform: impl FnOnce(PartitionState) -> PartitionState,
    ) {
        self.begin_current_target_edit(target, readmission);
        let TargetEdit { target, edit } = self
            .edits
            .remove(&target.partition_id())
            .expect("this writer owns the accepted resident target edit");
        let edit = edit.transform(transform);
        self.edits
            .insert(target.partition_id(), TargetEdit { target, edit });
    }

    pub(crate) fn install_target(&mut self, target: ResidentMirrorTarget) {
        let retained = target.retention();
        let id = target.partition_id();
        self.mirror
            .regions
            .insert(id, MirrorRegion::Resident(target));
        self.map_mut().insert(id, retained);
    }

    pub(crate) fn current_lease(
        &mut self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Head>>
    {
        self.mirror
            .admit_current_target(id)
            .ok()
            .flatten()
            .map(|target| target.current_lease())
    }

    /// Runtime pin/lifecycle metadata changes add no field meanings or history.
    pub(crate) fn partition_metadata_mut(
        &mut self,
        id: PartitionId,
    ) -> Option<&mut PartitionState> {
        if !self.edits.contains_key(&id) {
            let target = self.mirror.admit_current_target(id).ok().flatten()?;
            drop(self.map_mut().remove(&id));
            let (edit, copied) = target.begin_metadata_edit();
            if copied {
                self.copies.record_partition_copy(
                    edit.partition().entity_arena.slot_count(),
                    edit.partition().relation_arena.slot_count(),
                );
            }
            self.edits.insert(id, TargetEdit { target, edit });
        }
        self.edits
            .get_mut(&id)
            .map(|entry| entry.edit.partition_mut())
    }

    pub(crate) fn partition_mut(&mut self, id: PartitionId) -> Option<&mut PartitionState> {
        if !self.edits.contains_key(&id) {
            let target = self.mirror.admit_target(id).ok().flatten()?;
            if !self.begin_target_edit(&target) {
                return None;
            }
        }
        self.edits
            .get_mut(&id)
            .and_then(|entry| entry.edit.full_partition_mut())
    }

    pub(crate) fn resident_partitions_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut PartitionState> + use<'_> {
        for id in self.mirror.edition.partition_ids() {
            if let Ok(Some(target)) = self.mirror.admit_target(id) {
                self.begin_target_edit(&target);
            }
        }
        self.edits
            .values_mut()
            .filter_map(|entry| entry.edit.full_partition_mut())
    }

    #[cfg(test)]
    pub(crate) fn insert(&mut self, id: PartitionId, partition: PartitionState) {
        let retained = RelationalRegionRetention::resident(partition);
        let lease = retained
            .pin_lease()
            .expect("newly owned Full contents are resident");
        if let Some(entry) = self.edits.remove(&id) {
            entry
                .target
                .restore(ResidentRegionLease::Full(lease.clone()));
        } else if let Some(MirrorRegion::Resident(target)) = self.mirror.regions.get(&id) {
            target.restore(ResidentRegionLease::Full(lease.clone()));
        } else {
            self.mirror.regions.insert(
                id,
                MirrorRegion::Resident(ResidentMirrorTarget::new(lease.clone(), false)),
            );
        }
        self.map_mut()
            .insert(id, RelationalRegionRetention::from_lease(lease));
    }

    pub(crate) fn install(&mut self, partitions: PartitionMap) {
        self.finish_edits();
        *self.mirror = CurrentMirror::from_cells(partitions);
    }

    fn finish_edits(&mut self) {
        for (id, entry) in std::mem::take(&mut self.edits) {
            let lease = entry.edit.freeze();
            let retained = lease.retention();
            entry.target.restore(lease);
            self.map_mut().insert(id, retained);
        }
    }
}

impl Drop for PartitionEditionWriter<'_> {
    fn drop(&mut self) {
        self.finish_edits();
        self.lane.settle(self.instrumentation, self.copies);
    }
}

impl ResidentTargetEditor for PartitionEditionWriter<'_> {
    fn read_target(&self, target: &ResidentMirrorTarget) -> &PartitionState {
        match self.edits.get(&target.partition_id()) {
            Some(entry) => entry.edit.partition(),
            None => self
                .mirror
                .edition
                .partition(target.partition_id())
                .expect("accepted resident target remains in its writer's edition"),
        }
    }

    fn edit_target(&mut self, target: &ResidentMirrorTarget) -> &mut PartitionState {
        self.begin_target_edit(target);
        self.edits
            .get_mut(&target.partition_id())
            .expect("this writer owns the accepted target edit")
            .edit
            .partition_mut()
    }
}
