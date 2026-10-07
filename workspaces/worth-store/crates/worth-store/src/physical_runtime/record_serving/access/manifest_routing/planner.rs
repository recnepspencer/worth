use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, ManifestBlockReference,
    PersistedRecordIdentity, RecordArtifactFile, SegmentManifestBlockReference,
};

use super::{ManifestDiscoveryCounterSnapshot, ManifestLookupFailure, ManifestReader};

#[path = "planner/assignment.rs"]
mod assignment;
mod latest_publication;
mod leaf_updates;
mod rewrite;

use latest_publication::{surviving_directory_binding, surviving_latest_publication};
use rewrite::UpdatePlanner;

pub(in crate::physical_runtime::record_serving) struct ManifestPublicationPlan {
    pub(in crate::physical_runtime::record_serving) root: DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    pub(in crate::physical_runtime::record_serving) discovery: ManifestDiscoveryCounterSnapshot,
}

pub(in crate::physical_runtime::record_serving) struct RootManifestUpdateRequest<'updates> {
    pub(in crate::physical_runtime::record_serving) derived_updates:
        crate::physical_runtime::record_serving::planning::prepared_root_projection::DerivedRootUpdates,
    pub(in crate::physical_runtime::record_serving) release_head_effect:
        Option<&'updates worth_store_physical_format::PersistedReleaseCustodyHeadEffectV1>,
    pub(in crate::physical_runtime::record_serving) successor_generation: u64,
    pub(in crate::physical_runtime::record_serving) successor_capacity: u16,
    pub(in crate::physical_runtime::record_serving) free_space_checksum: u32,
    pub(in crate::physical_runtime::record_serving) free_space_root:
        Option<worth_store_physical_format::FreeSpaceBlockReference>,
    pub(in crate::physical_runtime::record_serving) segment_root:
        Option<SegmentManifestBlockReference>,
    pub(in crate::physical_runtime::record_serving) next_segment_block: u64,
    pub(in crate::physical_runtime::record_serving) placements:
        &'updates BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    pub(in crate::physical_runtime::record_serving) drops:
        &'updates BTreeSet<PersistedRecordIdentity>,
    pub(in crate::physical_runtime::record_serving) last_inline_record:
        Option<PersistedRecordIdentity>,
    pub(in crate::physical_runtime::record_serving) last_inline_segment:
        Option<worth_store_physical_format::SegmentGenerationCell>,
}

pub(in crate::physical_runtime::record_serving) fn plan_manifest_updates(
    reader: &ManifestReader<'_>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    current: &DurablePhysicalRootManifest,
    request: RootManifestUpdateRequest<'_>,
) -> Result<ManifestPublicationPlan, ManifestLookupFailure> {
    ManifestUpdatePlanning {
        reader,
        allocation,
        current,
        request,
    }
    .plan()
}

struct ManifestUpdatePlanning<'context, 'media, 'updates> {
    reader: &'context ManifestReader<'media>,
    allocation: &'context worth_store_buffer_pool::OperationAllocationGrant,
    current: &'context DurablePhysicalRootManifest,
    request: RootManifestUpdateRequest<'updates>,
}

impl ManifestUpdatePlanning<'_, '_, '_> {
    fn plan(self) -> Result<ManifestPublicationPlan, ManifestLookupFailure> {
        self.require_branchable_capacities()?;
        self.require_head_transition()?;
        if self
            .request
            .drops
            .iter()
            .any(|record| self.request.placements.contains_key(record))
            || unhandled_tail_drop(
                self.current.last_inline_record(),
                self.request.last_inline_record,
                self.request.drops,
                self.request.placements,
            )
        {
            return Err(ManifestLookupFailure::Damaged);
        }
        if self.request.successor_capacity != self.current.node_capacity() {
            if let Some(current_root) = self.current.routing_root() {
                return self.rebuild_for_successor_capacity(current_root);
            }
        }
        if let Some(plan) = self.incremental_when_packed()? {
            return Ok(plan);
        }
        let Some(current_root) = self.current.routing_root() else {
            return Err(ManifestLookupFailure::Damaged);
        };
        self.rebuild_for_successor_capacity(current_root)
    }

    fn require_branchable_capacities(&self) -> Result<(), ManifestLookupFailure> {
        let can_build_successor =
            super::super::super::planning::policy_units::manifest_capacity_can_branch(
                self.request.successor_capacity,
            );
        let can_read_current =
            super::super::super::planning::policy_units::manifest_capacity_can_branch(
                self.current.node_capacity(),
            );
        if can_build_successor && can_read_current {
            Ok(())
        } else {
            Err(ManifestLookupFailure::Damaged)
        }
    }

    fn require_head_transition(&self) -> Result<(), ManifestLookupFailure> {
        let Some(transition) = self.request.release_head_effect else {
            return Ok(());
        };
        if transition.source_root() != self.current.release_custody_head_root()
            || transition.source_next_block() != self.current.next_release_custody_head_block()
            || transition.result_root().generation() != self.request.successor_generation
            || transition.result_root().block() >= transition.result_next_block()
            || transition.result_next_block() < transition.source_next_block()
        {
            return Err(ManifestLookupFailure::Damaged);
        }
        Ok(())
    }

    fn rebuild_for_successor_capacity(
        self,
        current_root: ManifestBlockReference,
    ) -> Result<ManifestPublicationPlan, ManifestLookupFailure> {
        let Self {
            reader,
            allocation,
            current,
            request,
        } = self;
        let RootManifestUpdateRequest {
            derived_updates,
            release_head_effect,
            successor_generation,
            successor_capacity,
            free_space_checksum,
            free_space_root,
            segment_root,
            next_segment_block,
            placements: updates,
            drops,
            last_inline_record,
            last_inline_segment,
        } = request;
        let rebuilt = super::capacity_rebuild::rebuild_capacity(
            reader,
            allocation,
            super::capacity_rebuild::CapacityRebuildRequest {
                current_root,
                tree_identity: current.tree_identity(),
                successor_generation,
                successor_capacity,
                next_block: current.next_block(),
                updates: updates
                    .iter()
                    .map(|(record, placement)| (*record, *placement))
                    .collect(),
                drops: drops.clone(),
            },
        )?;
        let record_count = current
            .record_count()
            .checked_add(rebuilt.inserted)
            .and_then(|count| count.checked_sub(rebuilt.removed))
            .ok_or(ManifestLookupFailure::Damaged)?;
        let root = DurablePhysicalRootManifest::builder(
            successor_generation,
            current.tree_identity(),
            successor_capacity,
            free_space_checksum,
        )
        .record_count(record_count)
        .next_block(rebuilt.next_block)
        .next_segment_block(next_segment_block)
        .next_release_custody_head_block(
            release_head_effect.map_or(current.next_release_custody_head_block(), |transition| {
                transition.result_next_block()
            }),
        )
        .routing_root(rebuilt.root)
        .release_custody_head_root(
            release_head_effect.map_or(current.release_custody_head_root(), |transition| {
                Some(transition.result_root())
            }),
        )
        .segment_root(segment_root)
        .free_space_root(free_space_root)
        .tier_epoch_anchor(current.tier_epoch_anchor())
        .latest_blob_publication(surviving_latest_publication(
            current.latest_blob_publication(),
            derived_updates.latest_blob_publication,
            &drops,
        ))
        .latest_blob_quarantine(
            derived_updates
                .latest_blob_quarantine
                .or(current.latest_blob_quarantine()),
        )
        .derived_family_directory(surviving_directory_binding(
            current.derived_family_directory(),
            derived_updates.directory,
            &drops,
        ))
        .last_inline_record(last_inline_record)
        .last_inline_segment(last_inline_segment)
        .admit()
        .ok_or(ManifestLookupFailure::Damaged)?;
        let root = if current.requires_maintenance_protocol() {
            root.with_maintenance_protocol()
        } else {
            root
        };
        Ok(ManifestPublicationPlan {
            root,
            blocks: rebuilt.blocks,
            discovery: rebuilt.discovery,
        })
    }

    fn incremental_when_packed(
        &self,
    ) -> Result<Option<ManifestPublicationPlan>, ManifestLookupFailure> {
        let RootManifestUpdateRequest {
            derived_updates,
            release_head_effect,
            successor_generation,
            successor_capacity,
            free_space_checksum,
            free_space_root,
            segment_root,
            next_segment_block,
            placements: updates,
            drops,
            last_inline_record,
            last_inline_segment,
        } = self.request;
        let mut planner = UpdatePlanner {
            allocation: self.allocation,
            reader: self.reader,
            current: self.current,
            successor_generation,
            successor_capacity,
            next_block: self.current.next_block(),
            blocks: Vec::new(),
            discovery: ManifestDiscoveryCounterSnapshot::default(),
            inserted: 0,
            removed: 0,
        };
        let mut roots = match self.current.routing_root() {
            Some(root) => planner.rewrite(root, updates, drops)?,
            None => {
                if !drops.is_empty() {
                    return Err(ManifestLookupFailure::Damaged);
                }
                planner.inserted = updates.len() as u64;
                planner.write_leaves(updates.values().copied().collect())?
            }
        };
        leaf_updates::require_complete_drop(planner.removed, drops.len())?;
        while roots.len() > 1 {
            roots = planner.write_parent_level(roots)?;
        }
        let routing_root = roots.pop();
        let record_count = self
            .current
            .record_count()
            .checked_add(planner.inserted)
            .and_then(|count| count.checked_sub(planner.removed))
            .ok_or(ManifestLookupFailure::Damaged)?;
        let packed_level =
            worth_store_physical_format::required_tree_level(record_count, successor_capacity);
        if routing_root.map(ManifestBlockReference::level) != packed_level {
            return Ok(None);
        }
        let root = DurablePhysicalRootManifest::builder(
            successor_generation,
            self.current.tree_identity(),
            successor_capacity,
            free_space_checksum,
        )
        .record_count(record_count)
        .next_block(planner.next_block)
        .next_segment_block(next_segment_block)
        .next_release_custody_head_block(release_head_effect.map_or(
            self.current.next_release_custody_head_block(),
            |transition| transition.result_next_block(),
        ))
        .routing_root(routing_root)
        .release_custody_head_root(
            release_head_effect.map_or(self.current.release_custody_head_root(), |transition| {
                Some(transition.result_root())
            }),
        )
        .segment_root(segment_root)
        .free_space_root(free_space_root)
        .tier_epoch_anchor(self.current.tier_epoch_anchor())
        .latest_blob_publication(surviving_latest_publication(
            self.current.latest_blob_publication(),
            derived_updates.latest_blob_publication,
            drops,
        ))
        .latest_blob_quarantine(
            derived_updates
                .latest_blob_quarantine
                .or(self.current.latest_blob_quarantine()),
        )
        .derived_family_directory(surviving_directory_binding(
            self.current.derived_family_directory(),
            derived_updates.directory,
            drops,
        ))
        .last_inline_record(last_inline_record)
        .last_inline_segment(last_inline_segment)
        .admit()
        .ok_or(ManifestLookupFailure::Damaged)?;
        let root = if self.current.requires_maintenance_protocol() {
            root.with_maintenance_protocol()
        } else {
            root
        };
        Ok(Some(ManifestPublicationPlan {
            root,
            blocks: planner.blocks,
            discovery: planner.discovery,
        }))
    }
}

fn unhandled_tail_drop<T>(
    current_tail: Option<PersistedRecordIdentity>,
    successor_tail: Option<PersistedRecordIdentity>,
    drops: &BTreeSet<PersistedRecordIdentity>,
    placements: &BTreeMap<PersistedRecordIdentity, T>,
) -> bool {
    current_tail.is_some_and(|old_tail| {
        drops.contains(&old_tail)
            && !successor_tail.is_some_and(|new_tail| {
                new_tail != old_tail
                    && !drops.contains(&new_tail)
                    && placements.contains_key(&new_tail)
            })
    })
}

#[cfg(test)]
mod tail_drop_tests {
    use super::*;

    #[test]
    fn dropping_current_inline_tail_requires_distinct_routed_successor() {
        let old = PersistedRecordIdentity::new([7; 16], 1).unwrap();
        let new = PersistedRecordIdentity::new([7; 16], 2).unwrap();
        let drops = BTreeSet::from([old]);
        let mut placements = BTreeMap::new();
        assert!(unhandled_tail_drop(Some(old), None, &drops, &placements));
        assert!(unhandled_tail_drop(
            Some(old),
            Some(new),
            &drops,
            &placements
        ));
        placements.insert(new, ());
        assert!(!unhandled_tail_drop(
            Some(old),
            Some(new),
            &drops,
            &placements
        ));
        assert!(unhandled_tail_drop(
            Some(old),
            Some(old),
            &drops,
            &placements
        ));
        assert!(!unhandled_tail_drop(
            Some(old),
            None,
            &BTreeSet::new(),
            &placements
        ));
    }
}
