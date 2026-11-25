//! Retract one coupled chain's live segment to the image before its head (#119).
//!
//! A successor round supersedes its chain's earlier rounds: every record they
//! changed returns to its image at the chain's head (`B`), and every adjacency
//! delta they made is inverted, before the round applies as written. Records
//! and anchors another live member also touched keep that member's effect:
//! the successor guards make such overlap impossible for records, and an
//! anchor's structural revision and journal mark stay where another member's
//! delta stamped it.
use super::{AdjacencyDelta, AdjacencyDeltaKind};
use crate::contracts::identity::data::primitives::{KindId, PartitionId};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
use crate::contracts::transactions::data::primitives::RecordRef;
use std::collections::BTreeSet;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionMutationJournal;
use worth_relational_storage::internal_port::storage::overlay::working_state::retraction::AnchorRetraction;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use worth_relational_storage::internal_port::storage::substrate::SharedMap;

/// One adjacency anchor: partition, entity slot, and whether it is the
/// outgoing (source) side.
pub type Anchor = (PartitionId, usize, bool);

/// The chain head's image: its complete branch image and its journal.
pub struct RetractionBase<'a> {
    pub image: &'a dyn CurrentPartitionAccess,
    pub journal: &'a SharedMap<PartitionId, PartitionMutationJournal>,
}

/// What every other live member touched: anchors (by any adjacency delta or a
/// change of the anchor's own entity) and stamped anchor kinds.
#[derive(Default)]
pub struct OtherTouches {
    pub anchors: BTreeSet<Anchor>,
    pub kinds: BTreeSet<(Anchor, KindId)>,
}

impl OtherTouches {
    /// Note `delta` and, for an entity change, both anchors of `record`.
    pub fn note(&mut self, delta: Option<&AdjacencyDelta>, record: Option<&RecordRef>) {
        if let Some(delta) = delta {
            for anchor in anchors(delta) {
                self.anchors.insert(anchor);
                self.kinds.insert((anchor, delta.kind_id));
            }
        }
        if let Some(RecordRef::Entity(id)) = record {
            for outgoing in [true, false] {
                self.anchors
                    .insert((id.partition_id, id.slot_index(), outgoing));
            }
        }
    }
}

fn anchors(delta: &AdjacencyDelta) -> [Anchor; 2] {
    let (source, target) = match delta.kind {
        AdjacencyDeltaKind::Created { source, target }
        | AdjacencyDeltaKind::Deleted { source, target } => (source, target),
    };
    [
        (source.partition_id, source.slot_index(), true),
        (target.partition_id, target.slot_index(), false),
    ]
}

impl RetractionBase<'_> {
    fn marks_record(&self, record: &RecordRef) -> bool {
        let (partition, slot, entity) = match record {
            RecordRef::Entity(id) => (id.partition_id, id.slot_index(), true),
            RecordRef::Relation(id) => (id.partition_id, id.slot_index(), false),
        };
        self.journal.get(&partition).is_some_and(|journal| {
            if entity {
                journal.entity_slots.contains(&slot)
            } else {
                journal.relation_slots.contains(&slot)
            }
        })
    }

    fn marks_anchor(&self, (partition, slot, outgoing): Anchor) -> bool {
        self.journal.get(&partition).is_some_and(|journal| {
            if outgoing {
                journal.adjacency_slots.contains(&slot)
            } else {
                journal.reverse_adjacency_slots.contains(&slot)
            }
        })
    }
}

/// Retract the chain's live `records` and `deltas` over `state`.
/// O((|records| + |deltas|) · log n) plus the copied rows.
pub fn retract_chain_segment(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    base: &RetractionBase<'_>,
    records: &[RecordRef],
    deltas: &[AdjacencyDelta],
    others: &OtherTouches,
) -> Result<(), TransactionCommitError> {
    let denial =
        |cause: crate::contracts::execution::denial_cause::RelationalExecutionDenialCause| {
            TransactionCommitError::execution(cause.into())
        };
    let retraction = |anchor: Anchor, kind: KindId| AnchorRetraction {
        restore_revision: !others.kinds.contains(&(anchor, kind)),
        unmark: !base.marks_anchor(anchor) && !others.anchors.contains(&anchor),
    };
    for delta in deltas.iter().rev() {
        let [source, target] = anchors(delta);
        let anchors = [
            retraction(source, delta.kind_id),
            retraction(target, delta.kind_id),
        ];
        state
            .retract_adjacency_delta_for_request(delta, base.image, anchors, execution)
            .map_err(denial)?;
    }
    let mut seen = BTreeSet::new();
    for record in records {
        if !seen.insert(record) {
            continue;
        }
        state
            .retract_record_for_request(record, base.image, base.marks_record(record), execution)
            .map_err(denial)?;
        if let RecordRef::Entity(id) = record {
            for outgoing in [true, false] {
                let anchor = (id.partition_id, id.slot_index(), outgoing);
                if !base.marks_anchor(anchor) && !others.anchors.contains(&anchor) {
                    state.unmark_adjacency_anchor(id.partition_id, id.slot_index(), outgoing);
                }
            }
        }
    }
    Ok(())
}
