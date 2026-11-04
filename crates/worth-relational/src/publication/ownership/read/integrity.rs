//! Bidirectional current correspondence detects omitted, added and re-pointed facts.
use super::*;
use crate::mvcc::RequiredOutputRolePosture as Posture;

pub(super) fn validate(
    roster: &CurrentPublicationRoster,
    storage: &CurrentPublicationStorage,
    view: &InvariantStateView<'_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    for (id, catalog) in &roster.catalogs {
        checkpoint(
            context,
            (roster.catalogs.len() + roster.links.len() + 1) as u64,
        )?;
        if view.entity_metadata(catalog.scope).is_none()
            || roster
                .catalogs
                .iter()
                .filter(|(_, row)| row.scope == catalog.scope)
                .count()
                != 1
            || roster
                .links
                .iter()
                .filter(|link| {
                    link.kind == storage.scope_link
                        && link.source == *id
                        && link.target == catalog.scope
                        && link.publication == CommitId(0)
                })
                .count()
                != 1
        {
            return Err(corrupt(
                catalog
                    .routes
                    .first()
                    .map_or(CommitId(0), |route| route.publication),
                RecordRef::Entity(catalog.scope),
            ));
        }
        for (index, route) in catalog.routes.iter().enumerate() {
            checkpoint(context, (index + roster.publications.len() + 1) as u64)?;
            if route.meaning.source_scope != catalog.scope
                || catalog.routes[..index]
                    .iter()
                    .any(|old| old.meaning.same_route(&route.meaning))
                || roster
                    .publications
                    .iter()
                    .filter(|row| {
                        row.value.origin.meaning == route.meaning
                            && row.value.origin.commit.commit_id == route.publication
                            && row.digest == route.digest
                    })
                    .count()
                    != 1
            {
                return Err(corrupt(route.publication, RecordRef::Entity(catalog.scope)));
            }
        }
    }
    for row in &roster.publications {
        let origin = &row.value.origin;
        let publication = origin.commit.commit_id;
        let owner = origin.meaning.source_scope;
        checkpoint(
            context,
            (roster.publications.len()
                + roster.catalogs.len()
                + roster.links.len()
                + origin.members.len()
                + origin.relations.len()
                + 1) as u64,
        )?;
        let fail = || corrupt(publication, RecordRef::Entity(owner));
        if !origin.source_scope_owned
            || origin
                .members
                .iter()
                .all(|member| member.posture == Posture::Retire)
            || roster
                .publications
                .iter()
                .filter(|other| other.value.origin.meaning.same_route(&origin.meaning))
                .count()
                != 1
        {
            return Err(fail());
        }
        let Some((catalog_id, catalog)) = roster
            .catalogs
            .iter()
            .find(|(_, catalog)| catalog.scope == owner)
        else {
            return Err(fail());
        };
        if catalog
            .routes
            .iter()
            .filter(|route| {
                route.meaning == origin.meaning
                    && route.publication == publication
                    && route.digest == row.digest
            })
            .count()
            != 1
            || roster
                .links
                .iter()
                .filter(|link| {
                    link.kind == storage.publication_link
                        && link.source == row.entity
                        && link.target == *catalog_id
                        && link.publication == publication
                })
                .count()
                != 1
        {
            return Err(fail());
        }
        for member in origin
            .members
            .iter()
            .filter(|member| member.posture != Posture::Retire)
        {
            checkpoint(context, (roster.links.len() + 1) as u64)?;
            if view
                .entity_metadata(member.id)
                .is_none_or(|metadata| metadata.kind_id != member.kind)
                || roster
                    .links
                    .iter()
                    .filter(|link| {
                        link.kind == storage.member_link
                            && link.source == member.id
                            && link.target == row.entity
                            && link.publication == publication
                    })
                    .count()
                    != 1
            {
                return Err(corrupt(publication, RecordRef::Entity(member.id)));
            }
        }
        for edge in &origin.relations {
            checkpoint(
                context,
                (roster.edges.len() + roster.links.len() + 1) as u64,
            )?;
            let mut matching = roster
                .edges
                .iter()
                .filter(|(_, witness)| witness.publication == publication && witness.edge == *edge);
            let Some((witness_id, _)) = matching.next() else {
                return Err(corrupt(publication, RecordRef::Relation(edge.id)));
            };
            if matching.next().is_some() {
                return Err(corrupt(publication, RecordRef::Relation(edge.id)));
            }
            if view.relation_metadata(edge.id).is_none_or(|metadata| {
                metadata.kind_id != edge.kind
                    || metadata.source != edge.source
                    || metadata.target != edge.target
            }) || roster
                .links
                .iter()
                .filter(|link| {
                    link.kind == storage.edge_link
                        && link.source == *witness_id
                        && link.target == row.entity
                        && link.publication == publication
                })
                .count()
                != 1
            {
                return Err(corrupt(publication, RecordRef::Relation(edge.id)));
            }
        }
    }
    for (id, witness) in &roster.edges {
        checkpoint(
            context,
            (roster.publications.len() + roster.links.len() + 1) as u64,
        )?;
        let mut owners = 0;
        for row in &roster.publications {
            checkpoint(context, row.value.origin.relations.len() as u64)?;
            owners += usize::from(
                row.value.origin.commit.commit_id == witness.publication
                    && row.value.origin.relations.contains(&witness.edge)
                    && roster.links.iter().any(|link| {
                        link.kind == storage.edge_link
                            && link.source == *id
                            && link.target == row.entity
                            && link.publication == witness.publication
                    }),
            );
        }
        if owners != 1 {
            return Err(corrupt(
                witness.publication,
                RecordRef::Relation(witness.edge.id),
            ));
        }
    }
    for link in &roster.links {
        checkpoint(
            context,
            (roster.publications.len() + roster.catalogs.len() + roster.edges.len() + 1) as u64,
        )?;
        let valid = if link.kind == storage.scope_link {
            link.publication == CommitId(0)
                && roster
                    .catalogs
                    .iter()
                    .any(|(id, row)| *id == link.source && row.scope == link.target)
        } else if link.kind == storage.publication_link {
            roster.publications.iter().any(|row| {
                row.entity == link.source
                    && row.value.origin.commit.commit_id == link.publication
                    && roster.catalogs.iter().any(|(id, catalog)| {
                        *id == link.target && catalog.scope == row.value.origin.meaning.source_scope
                    })
            })
        } else {
            let Some(row) = roster.publications.iter().find(|row| {
                row.entity == link.target && row.value.origin.commit.commit_id == link.publication
            }) else {
                return Err(corrupt(link.publication, RecordRef::Relation(link.id)));
            };
            checkpoint(
                context,
                (row.value.origin.members.len() + roster.edges.len() + 1) as u64,
            )?;
            if link.kind == storage.member_link {
                row.value
                    .origin
                    .members
                    .iter()
                    .any(|member| member.id == link.source && member.posture != Posture::Retire)
            } else {
                roster.edges.iter().any(|(id, edge)| {
                    *id == link.source
                        && edge.publication == link.publication
                        && row.value.origin.relations.contains(&edge.edge)
                })
            }
        };
        if !valid {
            return Err(corrupt(link.publication, RecordRef::Relation(link.id)));
        }
    }
    Ok(())
}
