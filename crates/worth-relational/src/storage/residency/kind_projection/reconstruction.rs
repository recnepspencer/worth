use super::*;

/// Owned relations are reconstructed from record metadata. Peer relations use
/// only the projection authenticated by this region's canonical root leaf.
pub(in crate::storage::residency) fn rebuild(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
) -> Result<(), Cause> {
    let mut expected = BTreeMap::<RelationId, RegionRelationProjection>::new();
    for entry in projection {
        if expected.insert(entry.relation, entry.clone()).is_some() {
            return Err(Cause::Damaged);
        }
    }
    let historical = matches!(basis, AdjacencyKindBasis::Historical);
    if historical {
        // Discard every supplied claim about locally owned history, and derive
        // it again from the authenticated arena, including retired generations.
        expected.retain(|id, _| id.partition_id != partition.partition_id);
        project_owned_history(partition, &mut expected);
        if expected.values().cloned().collect::<Vec<_>>() != projection {
            return Err(Cause::IntegrityMismatch);
        }
    }
    install_projection(partition, &expected, basis)?;
    if !historical {
        for entry in expected
            .values()
            .filter(|entry| entry.relation.partition_id == partition.partition_id)
        {
            let view = partition
                .relation_arena
                .get_slot(entry.relation.slot_index())
                .ok_or(Cause::Damaged)?;
            if view.generation() != entry.relation.generation.0
                || view.kind_id() != Some(entry.kind)
            {
                return Err(Cause::IntegrityMismatch);
            }
            let mut local = BTreeMap::new();
            add_owned(
                partition,
                &mut local,
                entry.relation,
                entry.kind,
                view.extra().endpoints.as_ref().ok_or(Cause::Damaged)?,
            );
            if local.get(&entry.relation) != Some(entry) {
                return Err(Cause::IntegrityMismatch);
            }
        }
    }
    Ok(())
}

fn install_projection(
    partition: &mut PartitionState,
    expected: &BTreeMap<RelationId, RegionRelationProjection>,
    basis: AdjacencyKindBasis,
) -> Result<(), Cause> {
    let historical = matches!(basis, AdjacencyKindBasis::Historical);
    for (incoming, table) in [
        (false, &mut partition.adjacency),
        (true, &mut partition.reverse_adjacency),
    ] {
        for (_, adjacency) in table.iter_mut() {
            if historical {
                adjacency.reset_historical_kind_buckets();
            } else {
                adjacency.reset_kind_buckets();
            }
        }
        for entry in expected.values() {
            let slots = if incoming {
                &entry.incoming
            } else {
                &entry.outgoing
            };
            for &slot in slots {
                let adjacency = table.get_mut(slot).ok_or(Cause::IntegrityMismatch)?;
                if historical {
                    adjacency.index_historical_kind(entry.kind, entry.relation);
                } else {
                    adjacency.index_current_kind(entry.kind, entry.relation);
                }
            }
        }
        if !historical {
            for (_, adjacency) in table.iter() {
                let ids: BTreeSet<_> = adjacency.current_ids().iter().copied().collect();
                let indexed: BTreeSet<_> = adjacency
                    .checkpoint_kind_memberships(basis)
                    .into_iter()
                    .flat_map(|(_, ids)| ids)
                    .collect();
                if ids != indexed {
                    return Err(Cause::IntegrityMismatch);
                }
            }
        }
    }
    Ok(())
}

fn add_owned(
    partition: &PartitionState,
    target: &mut BTreeMap<RelationId, RegionRelationProjection>,
    relation: RelationId,
    kind: KindId,
    endpoints: &crate::storage::substrate::RelationEndpoints,
) {
    let entry = target
        .entry(relation)
        .or_insert_with(|| RegionRelationProjection {
            relation,
            kind,
            outgoing: BTreeSet::new(),
            incoming: BTreeSet::new(),
        });
    if endpoints.source.partition_id == partition.partition_id
        && partition
            .adjacency
            .get(endpoints.source.slot_index())
            .is_some()
    {
        entry.outgoing.insert(endpoints.source.slot_index());
    }
    if endpoints.target.partition_id == partition.partition_id
        && partition
            .reverse_adjacency
            .get(endpoints.target.slot_index())
            .is_some()
    {
        entry.incoming.insert(endpoints.target.slot_index());
    }
    if entry.outgoing.is_empty() && entry.incoming.is_empty() {
        target.remove(&relation);
    }
}

pub(in crate::storage::residency) fn canonical_history(
    partition: &PartitionState,
) -> Result<Vec<RegionRelationProjection>, Cause> {
    let mut expected = capture(partition, AdjacencyKindBasis::Historical)?
        .into_iter()
        .filter(|entry| entry.relation.partition_id != partition.partition_id)
        .map(|entry| (entry.relation, entry))
        .collect();
    project_owned_history(partition, &mut expected);
    Ok(expected.into_values().collect())
}

fn project_owned_history(
    partition: &PartitionState,
    expected: &mut BTreeMap<RelationId, RegionRelationProjection>,
) {
    for slot in partition.relation_arena.occupied_slots() {
        for metadata in partition
            .relation_arena
            .metadata_history_at(slot)
            .into_iter()
            .flatten()
        {
            add_owned(
                partition,
                expected,
                RelationId::new(partition.partition_id, slot as u64, metadata.generation),
                metadata.kind_id,
                &metadata.endpoints,
            );
        }
        if let Some(view) = partition.relation_arena.get_slot(slot)
            && let (Some(kind), Some(endpoints)) = (view.kind_id(), view.extra().endpoints.as_ref())
        {
            add_owned(
                partition,
                expected,
                RelationId::new(partition.partition_id, slot as u64, view.generation()),
                kind,
                endpoints,
            );
        }
    }
}
