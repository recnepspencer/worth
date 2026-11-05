//! Re-point a real claim while preserving row/catalog/presence pairing and framing.
use super::*;
pub(super) fn repoint(
    partition: &mut PartitionCheckpointImage,
    publication: CommitId,
    storage: &CurrentPublicationStorage,
) -> bool {
    let owner = partition.entity_arena.extra.iter().find_map(|extra| {
        let (p, text) = payload(
            extra.authoritative_aspect_state.as_ref(),
            crate::schema::current_publication_ownership::PUBLICATION,
        )?;
        if p != publication.0 {
            return None;
        }
        let row: record::Publication = serde_json::from_str(text).ok()?;
        Some(row.origin.meaning.source_scope)
    });
    let Some(owner) = owner else {
        return false;
    };
    for index in 0..partition.relation_arena.extra.len() {
        if partition.relation_arena.kind_ids[index] != Some(storage.member_link)
            || partition.relation_arena.lifecycle[index] != RecordLifecycleState::Live
        {
            continue;
        }
        let extra = &mut partition.relation_arena.extra[index];
        if payload(
            extra.authoritative_aspect_state.as_ref(),
            crate::schema::current_publication_ownership::LINK,
        )
        .is_none_or(|(p, _)| p != publication.0)
        {
            continue;
        }
        let Some(endpoints) = extra.endpoints.as_mut() else {
            continue;
        };
        let old = endpoints.source;
        endpoints.source = owner;
        let slot = partition
            .relation_arena
            .slots
            .get(index)
            .copied()
            .unwrap_or(index as u64);
        let link = RelationId::new(
            partition.partition_id,
            slot,
            partition.relation_arena.generations[index],
        );
        let version = partition.relation_arena.created_at[index];
        for meta in &mut partition.relation_arena.metadata_history[index] {
            meta.endpoints.source = owner;
        }
        for adjacency in &mut partition.adjacency {
            if adjacency.slot == old.local_slot.0 {
                adjacency.relations.retain(|id| *id != link);
            }
        }
        if let Some(adjacency) = partition
            .adjacency
            .iter_mut()
            .find(|row| row.slot == owner.local_slot.0)
        {
            adjacency.relations.push(link);
            if !adjacency
                .structural_revisions
                .iter()
                .any(|(kind, _)| *kind == storage.member_link)
            {
                adjacency
                    .structural_revisions
                    .push((storage.member_link, version));
            }
        } else {
            partition.adjacency.push(DurableAdjacencyEntry {
                slot: owner.local_slot.0,
                relations: vec![link],
                structural_revisions: vec![(storage.member_link, version)],
            });
        }
        return true;
    }
    false
}
