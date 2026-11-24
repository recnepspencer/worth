//! The relations of a successor round's own-output view (#277). Native
//! certified each chain relation whose live adjacency changed since the chain
//! head's pre-pin (`RelationalCoupledChainOverlay::relations`), exclusive to the
//! chain. An adjacency read is served as the current image minus the chain's
//! own effects: uncovered relations as they are now, covered relations as they
//! were at the head pre-pin.
use worth_relational::facade::{
    identity::{EntityId, KindId, RelationId},
    mvcc::RelationalCoupledOverlayRelation,
    runtime::{RelationReadRecord, RelationalAdjacencyDirection},
};

/// The chain's covered relation ids and the adjacency lists they touch, on
/// either live side, each sorted for O(log R) lookup.
#[derive(Clone, Default)]
pub struct OverlayRelations {
    covered: Vec<RelationId>,
    touched: Vec<(EntityId, KindId, u8)>,
}

impl OverlayRelations {
    pub fn from_chain(relations: &[RelationalCoupledOverlayRelation]) -> Self {
        let mut covered = relations
            .iter()
            .map(|relation| relation.id())
            .collect::<Vec<_>>();
        let mut touched = Vec::with_capacity(relations.len() * 4);
        for side in relations
            .iter()
            .flat_map(|relation| [relation.head(), relation.current()])
            .flatten()
        {
            touched.push((
                side.source(),
                side.kind(),
                rank(RelationalAdjacencyDirection::Outgoing),
            ));
            touched.push((
                side.target(),
                side.kind(),
                rank(RelationalAdjacencyDirection::Incoming),
            ));
        }
        covered.sort_unstable();
        covered.dedup();
        touched.sort_unstable();
        touched.dedup();
        Self { covered, touched }
    }

    pub fn covers(&self, relation: RelationId) -> bool {
        self.covered.binary_search(&relation).is_ok()
    }

    /// Whether a covered relation lies, on either side, on `anchor`'s
    /// `kind` list in `direction`: only then can the served list differ from
    /// the current one.
    pub fn touches(
        &self,
        anchor: EntityId,
        kind: KindId,
        direction: RelationalAdjacencyDirection,
    ) -> bool {
        self.touched
            .binary_search(&(anchor, kind, rank(direction)))
            .is_ok()
    }

    /// The served list: `current` minus covered relations, plus the covered
    /// relations of `head`, in Native adjacency order. O(|current| + |head|)
    /// lookups, then one sort of the result.
    pub fn merge<T>(
        &self,
        current: Vec<T>,
        head: Vec<T>,
        record: impl Fn(&T) -> (RelationId, EntityId, EntityId),
    ) -> Vec<T> {
        let mut served = current
            .into_iter()
            .filter(|item| !self.covers(record(item).0))
            .chain(head.into_iter().filter(|item| self.covers(record(item).0)))
            .collect::<Vec<_>>();
        served.sort_by_key(|item| order_key(record(item)));
        served.dedup_by_key(|item| record(item).0);
        served
    }
}

/// The served Native records of one adjacency list.
pub fn merge_records(
    relations: &OverlayRelations,
    current: Vec<RelationReadRecord>,
    head: Vec<RelationReadRecord>,
) -> Vec<RelationReadRecord> {
    relations.merge(current, head, |record| {
        (record.relation_id, record.source, record.target)
    })
}

/// Native adjacency order: source, target, then relation identity, each by
/// partition and slot (`relation_identity_order_key`).
fn order_key(
    (relation, source, target): (RelationId, EntityId, EntityId),
) -> (u32, u64, u32, u64, u32, u64) {
    (
        source.partition_value(),
        source.local_slot_value(),
        target.partition_value(),
        target.local_slot_value(),
        relation.partition_value(),
        relation.local_slot_value(),
    )
}

const fn rank(direction: RelationalAdjacencyDirection) -> u8 {
    match direction {
        RelationalAdjacencyDirection::Outgoing => 0,
        RelationalAdjacencyDirection::Incoming => 1,
    }
}
