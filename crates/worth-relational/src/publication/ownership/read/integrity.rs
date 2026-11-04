//! One indexed correspondence pass; each link and witness is consumed exactly once.
use super::*;
use crate::mvcc::RequiredOutputRolePosture as Posture;
use std::collections::HashMap;

pub(super) fn validate(
    roster: &CurrentPublicationRoster,
    storage: &CurrentPublicationStorage,
    view: &InvariantStateView<'_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let mut links = HashMap::new();
    let mut catalogs = HashMap::new();
    let mut publications = HashMap::new();
    let mut edges = HashMap::new();
    for link in &roster.links {
        checkpoint(context, 1)?;
        let key = (link.kind, link.source, link.target, link.publication);
        if links.insert(key, link.id).is_some() {
            return Err(corrupt(link.publication, RecordRef::Relation(link.id)));
        }
    }
    let take =
        |links: &mut HashMap<_, _>, kind, source, target, publication, subject: RecordRef| {
            links
                .remove(&(kind, source, target, publication))
                .map(|_| ())
                .ok_or_else(|| corrupt(publication, subject))
        };
    for (id, catalog) in &roster.catalogs {
        checkpoint(context, 1)?;
        if catalogs.insert(catalog.scope, (*id, catalog)).is_some() {
            return Err(damaged(&RecordRef::Entity(*id)));
        }
        if links
            .remove(&(storage.scope_link, *id, catalog.scope, CommitId(0)))
            .is_none()
        {
            // A scope catalog does not designate one publication. Do not invent
            // publication zero when its structural incidence is unavailable.
            return Err(damaged(&RecordRef::Entity(*id)));
        }
    }
    for row in roster.publications.iter().chain(&roster.codec_retired) {
        checkpoint(context, 1)?;
        let o = &row.value.origin;
        let key = (
            &o.meaning.family,
            o.meaning.source_scope,
            o.meaning.source_partition,
        );
        if publications.insert(key, row).is_some() {
            return Err(corrupt(
                o.commit.commit_id,
                RecordRef::Entity(o.meaning.source_scope),
            ));
        }
    }
    for (id, witness) in &roster.edges {
        checkpoint(context, 1)?;
        if edges
            .insert((witness.publication, witness.edge.id), (*id, witness))
            .is_some()
        {
            return Err(corrupt(
                witness.publication,
                RecordRef::Relation(witness.edge.id),
            ));
        }
    }
    let mut routes = HashMap::new();
    for (_, catalog) in &roster.catalogs {
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
            let row = publications
                .get(&key)
                .ok_or_else(|| corrupt(route.publication, RecordRef::Entity(catalog.scope)))?;
            if row.value.origin.meaning != route.meaning
                || row.value.origin.commit.commit_id != route.publication
                || row.digest != route.digest
            {
                return Err(corrupt(route.publication, RecordRef::Entity(catalog.scope)));
            }
        }
    }
    for row in &roster.publications {
        let o = &row.value.origin;
        let publication = o.commit.commit_id;
        let owner = o.meaning.source_scope;
        checkpoint(context, 1)?;
        let key = (&o.meaning.family, owner, o.meaning.source_partition);
        if view.entity_metadata(owner).is_none()
            || !o.source_scope_owned
            || o.members.iter().all(|m| m.posture == Posture::Retire)
            || !routes.contains_key(&key)
        {
            return Err(corrupt(publication, RecordRef::Entity(owner)));
        }
        let (catalog, _) = catalogs
            .get(&owner)
            .ok_or_else(|| corrupt(publication, RecordRef::Entity(owner)))?;
        take(
            &mut links,
            storage.publication_link,
            row.entity,
            *catalog,
            publication,
            RecordRef::Entity(owner),
        )?;
        for member in o.members.iter().filter(|m| m.posture != Posture::Retire) {
            checkpoint(context, 1)?;
            if view
                .entity_metadata(member.id)
                .is_none_or(|m| m.kind_id != member.kind)
            {
                return Err(corrupt(publication, RecordRef::Entity(member.id)));
            }
            take(
                &mut links,
                storage.member_link,
                member.id,
                row.entity,
                publication,
                RecordRef::Entity(member.id),
            )?;
        }
        for edge in &o.relations {
            checkpoint(context, 1)?;
            let (id, witness) = edges
                .remove(&(publication, edge.id))
                .ok_or_else(|| corrupt(publication, RecordRef::Relation(edge.id)))?;
            if witness.edge != *edge
                || view.relation_metadata(edge.id).is_none_or(|m| {
                    m.kind_id != edge.kind || m.source != edge.source || m.target != edge.target
                })
            {
                return Err(corrupt(publication, RecordRef::Relation(edge.id)));
            }
            take(
                &mut links,
                storage.edge_link,
                id,
                row.entity,
                publication,
                RecordRef::Relation(edge.id),
            )?;
        }
    }
    for row in &roster.codec_retired {
        checkpoint(context, 1)?;
        let o = &row.value.origin;
        let owner = o.meaning.source_scope;
        let key = (&o.meaning.family, owner, o.meaning.source_partition);
        if !routes.contains_key(&key) {
            return Err(corrupt(o.commit.commit_id, RecordRef::Entity(owner)));
        }
        let (catalog, _) = catalogs
            .get(&owner)
            .ok_or_else(|| corrupt(o.commit.commit_id, RecordRef::Entity(owner)))?;
        take(
            &mut links,
            storage.publication_link,
            row.entity,
            *catalog,
            o.commit.commit_id,
            RecordRef::Entity(owner),
        )?;
    }
    for (id, position) in &roster.positions {
        checkpoint(context, 1)?;
        if links
            .remove(&(
                storage.position_link,
                *id,
                position.meaning.source_scope,
                CommitId(0),
            ))
            .is_none()
        {
            return Err(corrupt(
                position.publication,
                RecordRef::Entity(position.meaning.source_scope),
            ));
        }
    }
    checkpoint(context, (edges.len() + links.len()) as u64)?;
    if let Some(((_, edge), (_, witness))) = edges.into_iter().min_by_key(|((_, edge), _)| *edge) {
        return Err(corrupt(witness.publication, RecordRef::Relation(edge)));
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
