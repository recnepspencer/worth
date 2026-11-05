//! Validate the selected rows and touched incidence; cold admission owns region-local pairing.
use super::*;
use crate::mvcc::RequiredOutputRolePosture as Posture;
use std::collections::{HashMap, HashSet};

pub(super) fn validate(
    roster: &CurrentPublicationRoster,
    storage: &CurrentPublicationStorage,
    view: &InvariantStateView<'_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let mut links = HashMap::new();
    let mut catalogs = HashMap::new();
    let mut publications = HashMap::new();
    let mut routes = HashMap::new();
    let mut route_rows = HashSet::new();
    let live = roster
        .publications
        .iter()
        .map(|row| row.entity)
        .collect::<HashSet<_>>();
    let mut members = HashSet::new();
    let mut edges = HashMap::new();
    let witnesses = roster
        .edges
        .iter()
        .map(|(id, witness)| (*id, witness))
        .collect::<HashMap<_, _>>();
    for link in &roster.links {
        checkpoint(context, 1)?;
        if links
            .insert(
                (link.kind, link.source, link.target, link.publication),
                link.id,
            )
            .is_some()
        {
            return Err(corrupt(link.publication, RecordRef::Relation(link.id)));
        }
    }
    let take =
        |links: &mut HashMap<_, _>, kind, source, target, publication, subject: RecordRef| {
            links
                .remove(&(kind, source, target, publication))
                .map(|_| ())
                .ok_or_else(|| {
                    if publication == CommitId(0) {
                        damaged(&subject)
                    } else {
                        corrupt(publication, subject)
                    }
                })
        };
    for (id, catalog) in &roster.catalogs {
        checkpoint(context, 1)?;
        if catalogs.insert(catalog.scope, (*id, catalog)).is_some() {
            return Err(damaged(&RecordRef::Entity(*id)));
        }
        take(
            &mut links,
            storage.scope_link,
            *id,
            catalog.scope,
            CommitId(0),
            RecordRef::Entity(*id),
        )?;
        for route in &catalog.routes {
            checkpoint(context, 1)?;
            let key = (
                &route.meaning.family,
                route.meaning.source_scope,
                route.meaning.source_partition,
            );
            if route.meaning.source_scope != catalog.scope || routes.insert(key, route).is_some() {
                return Err(corrupt(route.publication, RecordRef::Entity(catalog.scope)));
            }
        }
    }
    for row in roster.publications.iter().chain(&roster.codec_retired) {
        checkpoint(context, 1)?;
        if publications.insert(row.entity, row).is_some() {
            return Err(corrupt(
                row.value.origin.commit.commit_id,
                RecordRef::Entity(row.entity),
            ));
        }
        let o = &row.value.origin;
        let owner = o.meaning.source_scope;
        let publication = o.commit.commit_id;
        let (catalog, _payload) = catalogs
            .get(&owner)
            .ok_or_else(|| corrupt(publication, RecordRef::Entity(owner)))?;
        let key = (&o.meaning.family, owner, o.meaning.source_partition);
        route_rows.insert(key);
        let route = routes
            .get(&key)
            .ok_or_else(|| corrupt(publication, RecordRef::Entity(owner)))?;
        if !o.source_scope_owned || !record::route_matches(&row.value, &row.digest, route) {
            return Err(corrupt(publication, RecordRef::Entity(owner)));
        }
        take(
            &mut links,
            storage.publication_link,
            row.entity,
            *catalog,
            publication,
            RecordRef::Entity(owner),
        )?;
        if live.contains(&row.entity) {
            // The immutable row has to be decoded in full. Only selectors that
            // are actually touched require member metadata/incidence reads.
            for member in &o.members {
                checkpoint(context, 1)?;
                if member.posture != Posture::Retire {
                    members.insert((row.entity, member.id));
                }
                if member.posture != Posture::Retire && roster.reached_entities.contains(&member.id)
                {
                    if view
                        .entity_metadata(member.id)
                        .is_none_or(|m| m.kind_id != member.kind)
                    {
                        return Err(corrupt(publication, RecordRef::Entity(member.id)));
                    }
                    if !links.contains_key(&(
                        storage.member_link,
                        member.id,
                        row.entity,
                        publication,
                    )) {
                        return Err(corrupt(publication, RecordRef::Entity(member.id)));
                    }
                }
            }
        }
        for edge in &o.relations {
            checkpoint(context, 1)?;
            edges.insert((row.entity, edge.id), edge);
        }
    }
    for link in &roster.links {
        checkpoint(context, 1)?;
        if link.kind == storage.member_link || link.kind == storage.edge_link {
            let row = publications
                .get(&link.target)
                .ok_or_else(|| corrupt(link.publication, RecordRef::Relation(link.id)))?;
            let o = &row.value.origin;
            if link.publication != o.commit.commit_id {
                return Err(corrupt(link.publication, RecordRef::Relation(link.id)));
            }
            if link.kind == storage.member_link {
                if !members.contains(&(row.entity, link.source)) {
                    return Err(corrupt(link.publication, RecordRef::Entity(link.source)));
                }
            } else {
                let witness = witnesses
                    .get(&link.source)
                    .ok_or_else(|| corrupt(link.publication, RecordRef::Entity(link.source)))?;
                if edges
                    .get(&(row.entity, witness.edge.id))
                    .is_none_or(|edge| **edge != witness.edge)
                {
                    return Err(corrupt(
                        link.publication,
                        RecordRef::Relation(witness.edge.id),
                    ));
                }
            }
            take(
                &mut links,
                link.kind,
                link.source,
                link.target,
                link.publication,
                RecordRef::Relation(link.id),
            )?;
        }
    }
    for (_, catalog) in &roster.catalogs {
        if roster.reached_scopes.contains(&catalog.scope) {
            for route in &catalog.routes {
                checkpoint(context, 1)?;
                if !route_rows.contains(&(
                    &route.meaning.family,
                    route.meaning.source_scope,
                    route.meaning.source_partition,
                )) {
                    return Err(corrupt(route.publication, RecordRef::Entity(catalog.scope)));
                }
            }
        }
    }
    for (id, p) in &roster.positions {
        take(
            &mut links,
            storage.position_link,
            *id,
            p.meaning.source_scope,
            CommitId(0),
            RecordRef::Entity(p.meaning.source_scope),
        )?;
    }
    if let Some(((.., publication), id)) = links.into_iter().min_by_key(|(_, id)| *id) {
        return Err(if publication == CommitId(0) {
            damaged(&RecordRef::Relation(id))
        } else {
            corrupt(publication, RecordRef::Relation(id))
        });
    }
    Ok(())
}
