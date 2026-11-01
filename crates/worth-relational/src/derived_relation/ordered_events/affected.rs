//! Nonmonotone event changes schedule complete both-image successor cohorts.
use super::*;
use crate::authority::mutation::{AdjacencyDelta, AdjacencyDeltaKind};
use crate::derived_relation::affected::lifecycle::{self, Lifecycle};
use crate::derived_relation::DerivedRelationNode as Node;
use crate::identity::data::KindId;

pub(in crate::derived_relation) fn collect(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    recipe: Recipe,
    reverse: bool,
    adjacency: &[AdjacencyDelta],
    lifecycle: &[Lifecycle],
    before: &mut impl SourceRead,
    after: &mut impl SourceRead,
) -> Result<Acquisition, Denial> {
    let mut old = side(
        registry, definition, recipe, reverse, adjacency, lifecycle, before,
    )?;
    let new = side(
        registry, definition, recipe, reverse, adjacency, lifecycle, after,
    )?;
    for member in new.members {
        insert_member(after, &mut old.members, member)?;
    }
    super::super::support::append(after, &mut old.support, new.support)?;
    Ok(old)
}
fn side(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    recipe: Recipe,
    reverse: bool,
    adjacency: &[AdjacencyDelta],
    lifecycle: &[Lifecycle],
    read: &mut impl SourceRead,
) -> Result<Acquisition, Denial> {
    let mut combined = Acquisition {
        members: BTreeSet::new(),
        support: Vec::new(),
    };
    let mut source = EventSource::new(registry, definition, recipe, read);
    let (event_kind, row_kind) = domains(definition, recipe);
    let mut seeds = BTreeSet::new();
    let mut rows = BTreeSet::new();
    for delta in adjacency {
        source.read.checkpoint(1)?;
        let (event, row) = match delta.kind {
            AdjacencyDeltaKind::Created { source, target }
            | AdjacencyDeltaKind::Deleted { source, target } => (source, target),
        };
        for node in [recipe.predecessor, recipe.introduced, recipe.retired] {
            let Node::Stored { relation, .. } = definition.parts().nodes[node] else {
                unreachable!()
            };
            if relation != delta.kind_id {
                continue;
            }
            insert_member(source.read, &mut seeds, event.into())?;
            if node != recipe.predecessor {
                insert_member(source.read, &mut rows, row.into())?;
            }
        }
    }
    for event in lifecycle {
        source.read.checkpoint(1)?;
        if event.before == Some(event_kind) || event.after == Some(event_kind) {
            insert_member(source.read, &mut seeds, event.carrier.into())?;
        }
        if event.before == Some(row_kind) || event.after == Some(row_kind) {
            insert_member(source.read, &mut rows, event.carrier.into())?;
            if lifecycle::kind(source.read, event.carrier)? != Some(row_kind) {
                continue;
            }
            // Retired rows and currently excluded rows still seed scheduling.
            for node in [recipe.introduced, recipe.retired] {
                for at in source.targets(node, event.carrier, true)? {
                    insert_member(source.read, &mut seeds, at.into())?;
                }
            }
        }
    }
    let mut live_seeds = Vec::new();
    super::super::source_buffer::reserve(source.read, &mut live_seeds, seeds.len())?;
    for seed in seeds {
        let event = seed.entity().ok_or(Definition::InvalidMemberDomain)?;
        if lifecycle::kind(source.read, event)? != Some(event_kind) {
            continue;
        }
        chain::prefix(&mut source, event)?;
        live_seeds.push(event);
    }
    if reverse {
        // Only prefixes of actual changed events contribute these rows;
        // future introductions are not swept into every changed-row cohort.
        for event in std::mem::take(&mut source.valid_prefix) {
            for node in [recipe.introduced, recipe.retired] {
                for row in source.targets(node, event, false)? {
                    insert_member(source.read, &mut rows, row.into())?;
                }
            }
        }
        for row in rows {
            insert_member(source.read, &mut combined.members, row)?;
        }
    } else {
        for event in chain::successors(&mut source, &live_seeds, None, true)? {
            insert_member(source.read, &mut combined.members, event.into())?;
        }
    }
    super::super::support::append(source.read, &mut combined.support, source.support)?;
    Ok(combined)
}
fn domains(definition: &Handle, recipe: Recipe) -> (KindId, KindId) {
    let Node::Stored { from, to, .. } = definition.parts().nodes[recipe.introduced] else {
        unreachable!("registered literal event leaf")
    };
    (from, to)
}
