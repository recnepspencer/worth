//! Reach custody only through the touched records' current, kind-indexed adjacency.
use super::*;
use crate::storage::partition::AdjacencyDirection;
use std::collections::{BTreeSet, VecDeque};

#[allow(clippy::too_many_arguments)]
pub(super) fn read(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    storage: &CurrentPublicationStorage,
    view: &InvariantStateView<'_>,
    records: &[RecordRef],
    roster: &mut CurrentPublicationRoster,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let root = &basis.inner.root;
    let mut queue = VecDeque::new();
    let mut linked = VecDeque::new();
    let mut entities = BTreeSet::new();
    let mut expanded = BTreeSet::new();
    let mut relations = BTreeSet::new();
    for record in records {
        checkpoint(context, 1)?;
        let partition = match record {
            RecordRef::Entity(id) => id.partition_id,
            RecordRef::Relation(id) => id.partition_id,
        };
        root.demand_head_partition(execution, partition)
            .map_err(|e| crate::authority::commit::preparation::storage_read_denial(basis, e))?;
        match record {
            RecordRef::Entity(id) => queue.push_back((*id, true, false)),
            RecordRef::Relation(id) => {
                if let Some(row) = view.relation_metadata(*id) {
                    queue.push_back((row.source, false, false));
                    queue.push_back((row.target, false, false));
                }
            }
        }
    }
    while let Some((id, business, scope_only)) = queue.pop_front().or_else(|| {
        // Complete private custody first. A business edge to an already reached
        // scope does not turn that scope into a second explicit write selector.
        while let Some(id) = linked.pop_front() {
            if !entities.contains(&id) {
                return Some((id, false, false));
            }
        }
        None
    }) {
        checkpoint(context, 1)?;
        let first = entities.insert(id);
        if scope_only {
            if !first {
                continue;
            }
        } else if !expanded.insert(id) {
            continue;
        } else {
            retain::<EntityId>(&mut roster._funding)?;
        }
        if first {
            retain::<EntityId>(&mut roster._funding)?;
        }
        root.demand_head_partition(execution, id.partition_id)
            .map_err(|e| crate::authority::commit::preparation::storage_read_denial(basis, e))?;
        // New candidate identities have no basis row. A demanded committed region
        // cannot disappear; ordinary absent selectors do not assert record presence.
        let Some(head) = root.get_current_partition(id.partition_id) else {
            if !matches!(
                root.probe_head_partition(id.partition_id),
                crate::storage::residency::PartitionHeadProbe::Absent
            ) {
                return Err(damaged(&RecordRef::Entity(id)));
            }
            continue;
        };
        let Some(row) = head.entity_arena().get(&id).filter(|r| r.is_live()) else {
            continue;
        };
        let kind = row.kind_id();
        if [
            Some(storage.catalog_kind),
            Some(storage.publication_kind),
            Some(storage.edge_kind),
            Some(storage.position_kind),
        ]
        .contains(&kind)
        {
            entity(roster, view, id, kind.unwrap(), storage, context)?;
        }
        // Business edges are considered only at explicit input selectors (linked
        // owners). Private links then carry the complete ownership closure.
        if business {
            for edge in head
                .outgoing_candidates(id)
                .iter()
                .chain(head.incoming_candidates(id).iter())
            {
                checkpoint(context, 1)?;
                let Some(edge) = view.relation_metadata(*edge) else {
                    continue;
                };
                if !storage.metadata_kinds().contains(&edge.kind_id) {
                    retain::<(EntityId, EntityId)>(&mut roster._funding)?;
                    linked.push_back(edge.source);
                    linked.push_back(edge.target);
                }
            }
        }
        for direction in [AdjacencyDirection::Outgoing, AdjacencyDirection::Incoming] {
            let table = head.adjacency(direction);
            checkpoint(context, table.navigation_work_bound() as u64 + 1)?;
            let Some(set) = table.get(id.slot_index()) else {
                continue;
            };
            for kind in [
                storage.publication_link,
                storage.member_link,
                storage.edge_link,
                storage.scope_link,
                storage.position_link,
            ] {
                // A publication's scope identifies its catalog and independent
                // position. It does not instruct custody to ascend into a parent
                // publication that happens to own that same source record.
                if scope_only && ![storage.scope_link, storage.position_link].contains(&kind) {
                    continue;
                }
                checkpoint(context, set.navigation_work_bound() as u64 + 1)?;
                for link in set.kind_ids(kind).iter() {
                    checkpoint(context, 1)?;
                    if !relations.insert(*link) {
                        continue;
                    }
                    // Cross-partition incidence is indexed at its endpoint as well.
                    root.demand_head_partition(execution, link.partition_id)
                        .map_err(|e| {
                            crate::authority::commit::preparation::storage_read_denial(basis, e)
                        })?;
                    let row = view
                        .relation_metadata(*link)
                        .ok_or_else(|| damaged(&RecordRef::Relation(*link)))?;
                    if row.kind_id != kind || (row.source != id && row.target != id) {
                        return Err(damaged(&RecordRef::Relation(*link)));
                    }
                    let (publication, bytes) = view
                        .relation_aspect_state(*link)
                        .and_then(|s| {
                            record::fields(s, crate::schema::current_publication_ownership::LINK)
                        })
                        .ok_or_else(|| damaged(&RecordRef::Relation(*link)))?;
                    if !bytes.is_empty() {
                        return Err(payload::malformed(publication, &RecordRef::Relation(*link)));
                    }
                    retain::<LinkRow>(&mut roster._funding)?;
                    roster.links.push(LinkRow {
                        id: *link,
                        kind,
                        publication,
                        source: row.source,
                        target: row.target,
                    });
                    queue.push_back((row.source, false, false));
                    let scope = [storage.scope_link, storage.position_link].contains(&kind);
                    queue.push_back((row.target, false, scope));
                }
            }
        }
    }
    for (_, witness) in &roster.edges {
        checkpoint(context, 1)?;
        root.demand_head_partition(execution, witness.edge.id.partition_id)
            .map_err(|e| crate::authority::commit::preparation::storage_read_denial(basis, e))?;
    }
    // Scope-only contacts do not prove coverage for a later write to the source.
    roster.reached_entities = expanded;
    roster.reached_relations = relations;
    for row in &roster.publications {
        roster
            .reached_relations
            .extend(row.value.origin.relations.iter().map(|e| e.id));
    }
    Ok(())
}
fn entity(
    roster: &mut CurrentPublicationRoster,
    view: &InvariantStateView<'_>,
    id: EntityId,
    kind: KindId,
    storage: &CurrentPublicationStorage,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    use crate::schema::current_publication_ownership::{CATALOG, EDGE, POSITION, PUBLICATION};
    let aspect = if kind == storage.catalog_kind {
        CATALOG
    } else if kind == storage.publication_kind {
        PUBLICATION
    } else if kind == storage.edge_kind {
        EDGE
    } else {
        POSITION
    };
    let (publication, bytes) = view
        .entity_aspect_state(id)
        .and_then(|s| record::fields(s, aspect))
        .ok_or_else(|| damaged(&RecordRef::Entity(id)))?;
    let subject = RecordRef::Entity(id);
    if kind == storage.catalog_kind {
        retain::<(EntityId, record::Catalog)>(&mut roster._funding)?;
        let value = payload::decode(bytes, publication, &subject, &mut roster._funding, context)?;
        roster.catalogs.push((id, value));
    } else if kind == storage.publication_kind {
        let value: record::Publication =
            payload::decode(bytes, publication, &subject, &mut roster._funding, context)?;
        if value.origin.commit.commit_id != publication {
            return Err(corrupt(publication, subject));
        }
        use sha2::Digest;
        retain::<PublicationRow>(&mut roster._funding)?;
        roster.publications.push(PublicationRow {
            entity: id,
            value,
            digest: sha2::Sha256::digest(bytes).into(),
        });
    } else if kind == storage.edge_kind {
        let value: record::Edge =
            payload::decode(bytes, publication, &subject, &mut roster._funding, context)?;
        if value.publication != publication {
            return Err(corrupt(publication, subject));
        }
        retain::<(EntityId, record::Edge)>(&mut roster._funding)?;
        roster.edges.push((id, value));
    } else {
        let value: crate::history::data::current_route_presence::RoutePresence =
            payload::decode(bytes, publication, &subject, &mut roster._funding, context)?;
        if value.publication != publication {
            return Err(corrupt(publication, subject));
        }
        retain::<crate::history::data::current_route_presence::PresenceRow>(&mut roster._funding)?;
        roster.positions.push((id, value));
    }
    Ok(())
}
