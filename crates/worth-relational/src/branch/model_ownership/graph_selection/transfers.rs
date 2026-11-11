//! Select live union members and recorded inverse participants under their real owners.
use super::*;
#[path = "transfer_history.rs"]
mod history;

pub(super) struct RestorationBoundary {
    pub current: BTreeSet<EntityId>,
    pub selected: BTreeSet<EntityId>,
    pub transferred: BTreeMap<EntityId, EntityId>,
    pub participants: BTreeSet<EntityId>,
    pub _memory: Vec<ExecutionMemoryReservation>,
}
pub(super) fn prepare(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    entities: &RelationalOwnedModelEntityBoundary,
    installed: &RelationalModelOwnership,
) -> Result<RestorationBoundary, Denial> {
    let ((participants, current_edges), mut memory) = basis::read(
        native,
        execution,
        entities.current_basis(),
        context,
        |read| {
            let participants = history::participants(
                native,
                execution,
                entities.current_basis(),
                entities.model(),
                read,
            )?;
            let mut current_edges = BTreeMap::new();
            for &entity in &participants {
                if !in_union(entities, entity) {
                    read.retain()?;
                    current_edges.insert(entity, observation(read, installed, entity)?);
                }
            }
            Ok::<_, Denial>((participants, current_edges))
        },
    )?;
    let ((current, selected, transferred, participants), selected_memory) = basis::read(
        native,
        execution,
        entities.selected_basis(),
        context,
        |read| {
            let mut current = BTreeSet::new();
            let mut selected = BTreeSet::new();
            let mut participants = participants;
            let mut transferred = BTreeMap::new();
            for &entity in entities.current_entities() {
                read.retain()?;
                current.insert(entity);
            }
            for &entity in entities.selected_entities() {
                read.retain()?;
                selected.insert(entity);
            }
            for &entity in entities
                .current_entities()
                .difference(entities.selected_entities())
            {
                if read.kind(entity)?.is_some()
                    && let Some(owner) = installed.owner(read, entity)?
                {
                    read.retain()?;
                    selected.insert(entity);
                    read.retain()?;
                    transferred.insert(entity, owner);
                    read.retain()?;
                    participants.insert(entity);
                }
            }
            for (&entity, before) in &current_edges {
                let after = observation(read, installed, entity)?;
                read.charge(
                    before
                        .edges
                        .len()
                        .saturating_add(after.edges.len())
                        .saturating_add(1),
                )?;
                if before != &after {
                    read.retain()?;
                    current.insert(entity);
                    if let Some(owner) = after.owner {
                        read.retain()?;
                        selected.insert(entity);
                        read.retain()?;
                        transferred.insert(entity, owner);
                    }
                }
            }
            // D1178 publication versions may have different member identities.
            for _ in &selected {
                read.retain()?;
            }
            let seeds = selected.iter().copied().collect::<Vec<_>>();
            for source in seeds {
                for entity in super::super::traversal::walk(read, source, true, &[])? {
                    if !selected.contains(&entity) {
                        read.retain()?;
                        selected.insert(entity);
                    }
                    if !entities.selected_entities().contains(&entity)
                        && let Some(owner) = installed.owner(read, entity)?
                    {
                        if let std::collections::btree_map::Entry::Vacant(entry) =
                            transferred.entry(entity)
                        {
                            read.retain()?;
                            entry.insert(owner);
                        }
                        if !participants.contains(&entity) {
                            read.retain()?;
                            participants.insert(entity);
                        }
                    }
                }
            }
            Ok::<_, Denial>((current, selected, transferred, participants))
        },
    )?;
    memory.extend(selected_memory);
    let mut current = current;
    let mut participants = participants;
    let ((), current_memory) = basis::read(
        native,
        execution,
        entities.current_basis(),
        context,
        |read| {
            for &source in &selected {
                if read.kind(source)?.is_none() {
                    continue;
                }
                for entity in super::super::traversal::walk(read, source, true, &[])? {
                    if !current.contains(&entity) {
                        read.retain()?;
                        current.insert(entity);
                    }
                    if !entities.current_entities().contains(&entity)
                        && !participants.contains(&entity)
                    {
                        read.retain()?;
                        participants.insert(entity);
                    }
                }
            }
            Ok::<_, Denial>(())
        },
    )?;
    memory.extend(current_memory);
    Ok(RestorationBoundary {
        current,
        selected,
        transferred,
        participants,
        _memory: memory,
    })
}
fn incoming(
    read: &mut basis::BasisRead<'_, '_, '_, '_, '_, '_>,
    installed: &RelationalModelOwnership,
    entity: EntityId,
) -> Result<Vec<StructuralRelationRecord>, Denial> {
    let mut edges = Vec::new();
    if read.kind(entity)?.is_some() {
        for edge in read.relations(entity, false)? {
            if declared(read, installed, edge)? {
                read.retain()?;
                edges.push(edge);
            }
        }
    }
    read.charge(
        edges
            .len()
            .saturating_mul(edges.len().checked_ilog2().unwrap_or(0) as usize + 1),
    )?;
    edges.sort_unstable_by_key(|edge| edge.relation_id);
    Ok(edges)
}

#[derive(PartialEq, Eq)]
struct OwnershipObservation {
    edges: Vec<StructuralRelationRecord>,
    owner: Option<EntityId>,
}
fn observation(
    read: &mut basis::BasisRead<'_, '_, '_, '_, '_, '_>,
    installed: &RelationalModelOwnership,
    entity: EntityId,
) -> Result<OwnershipObservation, Denial> {
    let owner = if read.kind(entity)?.is_some() {
        installed.owner(read, entity)?
    } else {
        None
    };
    Ok(OwnershipObservation {
        owner,
        edges: incoming(read, installed, entity)?,
    })
}
