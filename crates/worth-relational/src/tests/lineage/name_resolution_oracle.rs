//! Independent brute-force resolution oracle: replays the raw committed log
//! up to one basis into plain maps, then answers by direct reachability over
//! the flat edge list. It shares no code or data structure with the index.

use std::collections::{BTreeMap, BTreeSet};

use crate::aggregate_identity::AggregateGroupLineageEvent as Group;
use crate::facade::history::CommitId;
use crate::facade::identity::LineageId;
use crate::facade::lineage::{
    LineageEventKind, LineageSubject, PersistentNameCommittedBatch, PersistentNameKey,
    PersistentNameResolution, PersistentNameResolutionDenial, PersistentNameSelection,
    PersistentNameTarget,
};

use super::name_index_fixtures::Answer;
use super::name_world::Name;

/// Everything visible at one basis, rebuilt from scratch.
#[derive(Default)]
pub(super) struct OracleView {
    born: BTreeSet<PersistentNameKey>,
    live: BTreeMap<PersistentNameKey, (PersistentNameTarget, BTreeSet<Name>)>,
    sides: BTreeMap<PersistentNameKey, Vec<u8>>,
    last_end: BTreeMap<PersistentNameKey, u64>,
    edges: Vec<(u64, PersistentNameKey, PersistentNameKey)>,
}

fn key(subject: LineageSubject, name: Name) -> PersistentNameKey {
    if subject == LineageSubject::Relation {
        PersistentNameKey::Relation(name)
    } else {
        PersistentNameKey::Entity(name)
    }
}

fn target(key: PersistentNameKey, holder: Holder, lineage: LineageId) -> PersistentNameTarget {
    match (key, holder) {
        (PersistentNameKey::Entity(name), Holder::Entity(entity)) => PersistentNameTarget::Entity {
            name,
            entity,
            lineage,
        },
        (PersistentNameKey::Relation(name), Holder::Relation(relation)) => {
            PersistentNameTarget::Relation {
                name,
                relation,
                lineage,
            }
        }
        _ => panic!("oracle saw a record of the wrong subject"),
    }
}

#[derive(Clone, Copy)]
enum Holder {
    Entity(crate::facade::identity::EntityId),
    Relation(crate::facade::identity::RelationId),
}

/// Replays every batch committed at or before `basis`.
pub(super) fn view_at(log: &[PersistentNameCommittedBatch], basis: u64) -> OracleView {
    let mut view = OracleView::default();
    let mut holders: BTreeMap<LineageId, Holder> = BTreeMap::new();
    let mut names: BTreeMap<LineageId, PersistentNameKey> = BTreeMap::new();
    for batch in log.iter().filter(|batch| batch.commit().0 <= basis) {
        let at = batch.commit().0;
        for (lineage, entity) in &batch.entity_holders {
            holders.insert(*lineage, Holder::Entity(*entity));
        }
        for (lineage, relation) in &batch.relation_holders {
            holders.insert(*lineage, Holder::Relation(*relation));
        }
        for event in &batch.lineage_events {
            match event.kind() {
                LineageEventKind::Create => {
                    let lineage = event.targets()[0];
                    if let Some(birth) = event.name_birth() {
                        let k = key(event.subject(), birth.identity());
                        view.born.insert(k);
                        let side = birth.parts().canonical_discriminator();
                        if !side.is_empty() {
                            view.sides.insert(k, side.to_vec());
                        }
                        view.live
                            .insert(k, (target(k, holders[&lineage], lineage), BTreeSet::new()));
                        names.insert(lineage, k);
                    }
                    if let Some(kept) = event.kept_name() {
                        let k = key(event.subject(), kept);
                        let carriers = view.live[&k].1.clone();
                        view.live
                            .insert(k, (target(k, holders[&lineage], lineage), carriers));
                        names.insert(lineage, k);
                    }
                }
                LineageEventKind::Restore => {
                    let lineage = event.targets()[0];
                    let restored = event.kept_name().expect("Restore carries kept_name");
                    let k = key(event.subject(), restored);
                    view.live
                        .insert(k, (target(k, holders[&lineage], lineage), BTreeSet::new()));
                    names.insert(lineage, k);
                }
                _ => {}
            }
        }
        for event in &batch.lineage_events {
            if matches!(
                event.kind(),
                LineageEventKind::Create | LineageEventKind::Restore
            ) {
                continue;
            }
            let successors: Vec<_> = event
                .targets()
                .iter()
                .filter_map(|l| names.get(l))
                .copied()
                .collect();
            for source in event.sources() {
                let Some(&held) = names.get(source) else {
                    continue;
                };
                if successors.contains(&held) {
                    continue;
                }
                view.live.remove(&held);
                view.last_end.insert(held, at);
                view.edges
                    .extend(successors.iter().map(|next| (at, held, *next)));
            }
        }
        let groups = &batch.group_events;
        for event in groups {
            if let Group::Born { group } | Group::Revived { group } = *event {
                let k = PersistentNameKey::Group(group);
                view.born.insert(k);
                view.live
                    .insert(k, (PersistentNameTarget::Group { group }, BTreeSet::new()));
            }
        }
        for event in groups {
            if let Group::Retired { group } = *event {
                let k = PersistentNameKey::Group(group);
                view.live.remove(&k);
                view.last_end.insert(k, at);
            }
        }
        for event in groups {
            match *event {
                Group::SplitFrom { fragment, source } => view.edges.push((
                    at,
                    PersistentNameKey::Group(source),
                    PersistentNameKey::Group(fragment),
                )),
                Group::MergedInto { absorbed, keeper } => view.edges.push((
                    at,
                    PersistentNameKey::Group(absorbed),
                    PersistentNameKey::Group(keeper),
                )),
                _ => {}
            }
        }
        for (k, carriers) in &batch.carriers {
            view.live.get_mut(k).expect("carriers name a live name").1 = carriers.clone();
        }
    }
    view
}

/// Direct reachability: a dead name continues only through the edges
/// recorded at the commit where it last ended.
pub(super) fn answer(
    view: &OracleView,
    held: PersistentNameKey,
    selection: &PersistentNameSelection,
) -> Answer {
    let wildcard = match selection {
        PersistentNameSelection::BoundedByCarriers { carriers } => carriers.is_empty(),
        PersistentNameSelection::KeepSide { discriminator } => discriminator.is_empty(),
        _ => false,
    };
    if wildcard {
        return Err(PersistentNameResolutionDenial::EmptySelectionFilter);
    }
    if !view.born.contains(&held) {
        return Err(PersistentNameResolutionDenial::UnknownName { name: held });
    }
    let mut live = BTreeSet::new();
    let mut seen = BTreeSet::from([held]);
    let mut pending = vec![held];
    let mut retired_at = 0;
    while let Some(name) = pending.pop() {
        if view.live.contains_key(&name) {
            live.insert(name);
            continue;
        }
        let ended = view.last_end[&name];
        retired_at = retired_at.max(ended);
        for (at, from, to) in &view.edges {
            if *at == ended && *from == name && seen.insert(*to) {
                pending.push(*to);
            }
        }
    }
    let live: Vec<PersistentNameKey> = live.into_iter().collect();
    if live.is_empty() {
        return Ok(PersistentNameResolution::Retired {
            at_commit: CommitId(retired_at),
        });
    }
    let target = |k: &PersistentNameKey| view.live[k].0;
    let chosen: Vec<PersistentNameKey> = match selection {
        PersistentNameSelection::Exact => live.clone(),
        PersistentNameSelection::AllDescendants if live.len() > 1 => {
            return Ok(PersistentNameResolution::Plural(
                live.iter().map(target).collect(),
            ));
        }
        PersistentNameSelection::AllDescendants => live.clone(),
        PersistentNameSelection::BoundedByCarriers { carriers } => live
            .iter()
            .filter(|k| carriers.iter().all(|c| view.live[k].1.contains(c)))
            .copied()
            .collect(),
        PersistentNameSelection::KeepSide { discriminator } => live
            .iter()
            .filter(|k| view.sides.get(k) == Some(discriminator))
            .copied()
            .collect(),
    };
    match chosen.len() {
        0 => Err(PersistentNameResolutionDenial::NoMatchingCandidate {
            live_candidates: live,
        }),
        1 => Ok(PersistentNameResolution::Exact(target(&chosen[0]))),
        _ => Err(PersistentNameResolutionDenial::Ambiguous { candidates: chosen }),
    }
}
