//! Decode one explicitly reached Native current-fact row.
use super::*;
pub(super) fn load(
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
