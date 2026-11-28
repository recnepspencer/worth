//! One exclusive writer preserves current target metadata and old editions.
pub mod resident_entry;
pub use resident_entry::ResidentPartitionEntry;
use resident_entry::ResidentTargetEditor;
use std::collections::BTreeMap;
use std::sync::RwLockWriteGuard;

use crate::identity::data::PartitionId;
use crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;
use crate::storage::overlay::{CurrentPartitionAccess, PartitionState};
use crate::storage::residency::{RelationalRegionRetention, ResidentRegionLease};
use crate::storage::residency::{ResidentHistoryReadmission, ResidentRegionEdit};
use std::sync::Mutex;

use super::edition_copy_lane::{PartitionCopyTally, PartitionEditionCopyLane};
use super::{CurrentMirror, MirrorRegion, PartitionMap, ResidentMirrorTarget};

pub struct PartitionEditionWriter<'subsystem> {
    mirror: RwLockWriteGuard<'subsystem, CurrentMirror>,
    copy_counters: &'subsystem Mutex<RuntimeComplexityCounters>,
    lane: PartitionEditionCopyLane,
    copies: PartitionCopyTally,
    edits: BTreeMap<PartitionId, TargetEdit>,
}

struct TargetEdit {
    target: ResidentMirrorTarget,
    edit: ResidentRegionEdit,
}

/// Exclusive, load-free access to a mirror whose entire edition is Full.
/// Unedited partitions release their borrowed Full leases on drop without
/// promoting mirror regions or pinning mirror `owner` slots.
pub struct ResidentPartitionEditionWriter<'subsystem> {
    writer: PartitionEditionWriter<'subsystem>,
    leases: BTreeMap<
        PartitionId,
        crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Full>,
    >,
    partition_ids: Vec<PartitionId>,
}

struct ResidentEditionEditor<'editor, 'subsystem> {
    writer: &'editor mut PartitionEditionWriter<'subsystem>,
    leases: &'editor mut BTreeMap<
        PartitionId,
        crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Full>,
    >,
}

impl ResidentPartitionEditionWriter<'_> {
    pub fn visit_partitions(
        &mut self,
        mut visit: impl FnMut(
            ResidentPartitionEntry<'_>,
        ) -> Result<
            (),
            crate::execution::denial_cause::RelationalExecutionDenialCause,
        >,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        for &id in &self.partition_ids {
            let mut editor = ResidentEditionEditor {
                writer: &mut self.writer,
                leases: &mut self.leases,
            };
            visit(ResidentPartitionEntry {
                id,
                writer: &mut editor,
            })?;
        }
        Ok(())
    }
}

impl<'subsystem> PartitionEditionWriter<'subsystem> {
    pub(super) fn new(
        mirror: RwLockWriteGuard<'subsystem, CurrentMirror>,
        copy_counters: &'subsystem Mutex<RuntimeComplexityCounters>,
        lane: PartitionEditionCopyLane,
    ) -> Self {
        Self {
            mirror,
            copy_counters,
            lane,
            copies: PartitionCopyTally::default(),
            edits: BTreeMap::new(),
        }
    }

    pub(super) fn accept_resident(
        self,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        leases: BTreeMap<
            PartitionId,
            crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Full>,
        >,
    ) -> Result<ResidentPartitionEditionWriter<'subsystem>, Self> {
        if leases.len() != self.mirror.edition.len() {
            return Err(self);
        }
        let partition_ids = {
            let mut ids = Vec::with_capacity(leases.len());
            let mut matched = true;
            for (id, region) in self.mirror.edition.cells() {
                let Some(lease) = leases.get(&id) else {
                    matched = false;
                    break;
                };
                if !std::sync::Arc::ptr_eq(lease.cell(), region.cell())
                    || !symbols.region_symbols_admitted(lease.symbol_admission_identity())
                {
                    matched = false;
                    break;
                }
                ids.push(id);
            }
            matched.then_some(ids)
        };
        let Some(partition_ids) = partition_ids else {
            return Err(self);
        };
        Ok(ResidentPartitionEditionWriter {
            writer: self,
            leases,
            partition_ids,
        })
    }

    fn map_mut(&mut self) -> &mut PartitionMap {
        self.mirror.edition.map_mut()
    }

    fn begin_target_edit(&mut self, target: &ResidentMirrorTarget) -> bool {
        let id = target.partition_id();
        if let Some(entry) = self.edits.get_mut(&id) {
            return entry.edit.full_partition_mut().is_some();
        }
        if !target.is_full() {
            return false;
        }
        debug_assert!(
            /* Proof: PartitionEditionWriter retains the exclusive RwLockWriteGuard<CurrentMirror>. Resident targets are selected from that same mirror; every edit is retained in this writer until finish_edits restores it, before the write guard can be released. */
            matches!(self.mirror.regions.get(&id),
            Some(MirrorRegion::Resident(current)) if current.same_target(target))
        );
        // The cached edition is an owner projection, not another observer.
        // An external edition retains its old map and payload independently.
        self.map_mut().remove(&id);
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
        let active = match self.mirror.regions.get(&id) {
            Some(MirrorRegion::Resident(current)) => current.clone(),
            Some(MirrorRegion::AwaitingResidentTarget(_)) => {
                self.mirror
                    .regions
                    .insert(id, MirrorRegion::Resident(target.clone()));
                target.clone()
            }
            None => unreachable!(
                /* Proof: PartitionEditionWriter holds the exclusive RwLockWriteGuard<CurrentMirror>; prepared existing targets are built only from partitions present in CurrentMirror, and partitions are never removed from CurrentMirror. */
                "prepared existing target exists in mirror"
            ),
        };
        self.map_mut().remove(&id);
        let (edit, copied) = active.begin_current_edit(readmission);
        if copied {
            self.copies.record_partition_copy(
                edit.partition().entity_arena.slot_count(),
                edit.partition().relation_arena.slot_count(),
            );
        }
        self.edits.insert(
            id,
            TargetEdit {
                target: active,
                edit,
            },
        );
    }

    pub fn transform_target(
        &mut self,
        target: &ResidentMirrorTarget,
        readmission: ResidentHistoryReadmission,
        transform: impl FnOnce(PartitionState) -> PartitionState,
    ) {
        self.begin_current_target_edit(target, readmission.clone());
        let TargetEdit { target, edit } = self
            .edits
            .remove(&target.partition_id())
            .expect(/* Proof: Accepted target creation registers this partition edit in its owning edition writer; the private target retains that writer identity and extraction consumes the registered edit once. */ "this writer owns the accepted resident target edit");
        let edit = edit.transform(readmission, transform);
        self.edits
            .insert(target.partition_id(), TargetEdit { target, edit });
    }

    pub fn install_target(&mut self, target: ResidentMirrorTarget) {
        let retained = target.retention();
        let id = target.partition_id();
        self.mirror
            .regions
            .insert(id, MirrorRegion::Resident(target));
        self.map_mut().insert(id, retained);
    }

    pub fn current_lease(
        &mut self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Head>>
    {
        self.mirror
            .admit_current_target(id)
            .ok()
            .flatten()
            .and_then(|target| target.current_lease())
    }

    /// Runtime pin/lifecycle metadata changes add no field meanings or history.
    pub fn partition_metadata_mut(&mut self, id: PartitionId) -> Option<&mut PartitionState> {
        if !self.edits.contains_key(&id) {
            let target = self.mirror.admit_current_target(id).ok().flatten()?;
            self.map_mut().remove(&id);
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

    pub fn partition_mut(&mut self, id: PartitionId) -> Option<&mut PartitionState> {
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

    pub fn resident_partitions_mut(
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

    pub(crate) fn insert(&mut self, id: PartitionId, partition: PartitionState) {
        let lease =
            crate::storage::residency::RelationalRegionResidency::resident_uncommitted(partition);
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
            .insert(id, RelationalRegionRetention::mirror_owned(lease));
    }

    pub fn install(&mut self, partitions: PartitionMap) {
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
        self.lane.settle(self.copy_counters, self.copies);
    }
}

impl ResidentTargetEditor for ResidentEditionEditor<'_, '_> {
    fn read_target(
        &self,
        id: PartitionId,
    ) -> Result<&PartitionState, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        if let Some(entry) = self.writer.edits.get(&id) {
            return Ok(entry.edit.partition());
        }
        self.leases
            .get(&id)
            .map(|lease| lease.full_view().partition())
            .ok_or({
                crate::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(
                    crate::storage::data::RelationalStorageUnavailable {
                        subject: crate::storage::data::RelationalStorageSubject::Region(id),
                        cause:
                            crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
                    },
                )
            })
    }

    fn edit_target(&mut self, id: PartitionId) -> &mut PartitionState {
        if !self.writer.edits.contains_key(&id) {
            let lease = self.leases.remove(&id).expect(
                /* Proof: accept_resident verifies every partition_id has a live Full lease in self.leases, and edits.contains_key guards against removing it more than once. */
                "accepted resident partition holds a Full lease",
            );
            let target = self.writer.mirror.admit_target_with_lease(id, lease).expect(
                /* Proof: PartitionEditionWriter holds the exclusive RwLockWriteGuard<CurrentMirror> since accept_resident verified id exists in mirror. */
                "accepted resident partition exists in mirror",
            );
            self.writer.begin_target_edit(&target);
        }
        self.writer
            .edits
            .get_mut(&id)
            .expect(
                /* Proof: begin_target_edit registers the Full edit for id above before returning. */
                "this writer owns the accepted target edit",
            )
            .edit
            .partition_mut()
    }
}

pub mod try_maintenance;
pub use try_maintenance::TryPartitionMutationDenied;

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait PartitionEditionWriterFixturePort<'subsystem>: Sized {
    fn install_partition_for_test(&mut self, id: PartitionId, partition: PartitionState);
}

impl<'subsystem> PartitionEditionWriterFixturePort<'subsystem>
    for PartitionEditionWriter<'subsystem>
{
    fn install_partition_for_test(&mut self, id: PartitionId, partition: PartitionState) {
        self.insert(id, partition)
    }
}
