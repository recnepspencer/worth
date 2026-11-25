//! Return records and adjacency of a superseded coupled round to the image
//! before its chain's head (`base`, a complete branch image), #119.
//!
//! The overlay is sparse over the selected basis: a slot it holds shadows the
//! basis, and the journal names the slots the candidate publishes. A record
//! `base` holds takes `base`'s exact row and keeps its journal mark only when
//! the candidate had already changed it before the head (`base_journaled`); a
//! record `base` lacks leaves the overlay, so it is absent again.
use super::WorkingState;
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::storage::partition::AdjacencyDirection;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::authority::mutation::adjacency_delta::{
    AdjacencyDelta, AdjacencyDeltaKind,
};

/// How one anchor of a retracted adjacency delta returns to `base`.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnchorRetraction {
    /// Restore the kind's structural revision to `base`'s: no other live
    /// member stamped this anchor and kind.
    pub restore_revision: bool,
    /// Forget the anchor's journal mark: `base` did not mark it and no other
    /// live member touched it.
    pub unmark: bool,
}

impl WorkingState {
    /// Retract `record` to its image in `base`. O(log n + row payload).
    pub fn retract_record_for_request(
        &mut self,
        record: &RecordRef,
        base: &dyn CurrentPartitionAccess,
        base_journaled: bool,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        request.consult()?;
        let (partition_id, slot, entity) = match record {
            RecordRef::Entity(id) => (id.partition_id, id.slot_index(), true),
            RecordRef::Relation(id) => (id.partition_id, id.slot_index(), false),
        };
        let base_partition = base.get_current_partition(partition_id);
        let restored = if let Some(overlay) = self.partitions.get_mut(&partition_id) {
            let base = base_partition.map(|view| view.partition_state());
            match (entity, base) {
                (true, Some(base)) if base.entity_arena.physical_index(slot).is_some() => overlay
                    .entity_arena
                    .restore_slot_from_base_for_request(&base.entity_arena, slot),
                (false, Some(base)) if base.relation_arena.physical_index(slot).is_some() => {
                    overlay
                        .relation_arena
                        .restore_slot_from_base_for_request(&base.relation_arena, slot)
                }
                (true, _) => !overlay
                    .entity_arena
                    .retract_uncommitted_slot_for_request(slot),
                (false, _) => !overlay
                    .relation_arena
                    .retract_uncommitted_slot_for_request(slot),
            }
        } else {
            false
        };
        if let Some(journal) = self.mutation_journal.get_mut(&partition_id) {
            let slots = if entity {
                &mut journal.entity_slots
            } else {
                &mut journal.relation_slots
            };
            if restored && base_journaled {
                slots.insert_for_request(slot, request)?;
            } else {
                slots.remove(slot);
            }
        }
        Ok(())
    }

    /// Invert `delta` on both anchors, with `base` supplying the structural
    /// revision an anchor returns to. O(log degree) per anchor.
    pub fn retract_adjacency_delta_for_request(
        &mut self,
        delta: &AdjacencyDelta,
        base: &dyn CurrentPartitionAccess,
        anchors: [AnchorRetraction; 2],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        let (source, target, created) = match delta.kind {
            AdjacencyDeltaKind::Created { source, target } => (source, target, true),
            AdjacencyDeltaKind::Deleted { source, target } => (source, target, false),
        };
        for ((anchor, direction), retraction) in [
            (source, AdjacencyDirection::Outgoing),
            (target, AdjacencyDirection::Incoming),
        ]
        .into_iter()
        .zip(anchors)
        {
            let (partition_id, slot) = (anchor.partition_id, anchor.slot_index());
            let revision = retraction.restore_revision.then(|| {
                base.get_current_partition(partition_id).and_then(|view| {
                    direction
                        .table(view.partition_state())
                        .get(slot)
                        .and_then(|set| set.structural_revision(delta.kind_id))
                })
            });
            if let Some(overlay) = self.partitions.get_mut(&partition_id) {
                let table = match direction {
                    AdjacencyDirection::Outgoing => &mut overlay.adjacency,
                    AdjacencyDirection::Incoming => &mut overlay.reverse_adjacency,
                };
                if let Some(set) = table.get_mut_for_request(slot, request)? {
                    set.retract_delta_for_request(
                        delta.kind_id,
                        delta.relation_id,
                        created,
                        revision,
                        request,
                    )?;
                }
            }
            if retraction.unmark
                && let Some(journal) = self.mutation_journal.get_mut(&partition_id)
            {
                match direction {
                    AdjacencyDirection::Outgoing => journal.adjacency_slots.remove(slot),
                    AdjacencyDirection::Incoming => journal.reverse_adjacency_slots.remove(slot),
                };
            }
        }
        Ok(())
    }

    /// Forget one adjacency anchor's journal mark: it holds its basis image.
    pub fn unmark_adjacency_anchor(
        &mut self,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
        outgoing: bool,
    ) {
        if let Some(journal) = self.mutation_journal.get_mut(&partition_id) {
            if outgoing {
                journal.adjacency_slots.remove(slot);
            } else {
                journal.reverse_adjacency_slots.remove(slot);
            }
        }
    }
}
