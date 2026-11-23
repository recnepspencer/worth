//! Commit-local persistent-name birth extraction from a sealed commit's own
//! canonical lineage artifact and record allocations (O(commit events), zero
//! runtime lock acquisitions, immune to post-commit retirement races).

use std::collections::BTreeMap;

use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::lineage::data::{
    LineageSubject, PersistentNameBirthRecord, PersistentNameHolder,
};

pub(crate) fn extract_committed_name_births(
    envelope: &CanonicalCommitEnvelope,
) -> Vec<(PersistentNameHolder, PersistentNameBirthRecord)> {
    let events = envelope.lineage_events();
    if !events.iter().any(|event| event.name_birth().is_some()) {
        return Vec::new();
    }
    let entity_by_lineage: BTreeMap<LineageId, EntityId> =
        crate::durability::authority::lineage_allocation::entity_allocation_bindings(
            &crate::durability::authority::lineage_allocation::CommitAllocationView::of_envelope(
                envelope,
            ),
        )
        .unwrap_or_default()
        .into_iter()
        .collect();
    let relation_by_lineage: BTreeMap<LineageId, RelationId> = envelope
        .lineage_relation_lineages()
        .iter()
        .map(|&(relation_id, lineage_id)| (lineage_id, relation_id))
        .collect();
    let mut births = Vec::new();
    for event in events {
        let Some(birth) = event.name_birth() else {
            continue;
        };
        let Some(&lineage_id) = event.targets().first() else {
            continue;
        };
        let holder = match event.subject() {
            LineageSubject::Entity => entity_by_lineage
                .get(&lineage_id)
                .copied()
                .map(PersistentNameHolder::Entity),
            LineageSubject::Relation => relation_by_lineage
                .get(&lineage_id)
                .copied()
                .map(PersistentNameHolder::Relation),
        };
        if let Some(holder) = holder {
            births.push((holder, birth.clone()));
        }
    }
    births
}
