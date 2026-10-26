//! Complete orbit answers cannot silently become filtered or reordered sets.
use super::{ApplicationQueryDefinitionDenial as Denial, WorthQueryPortableApplicationQueryParts};
use crate::application_query::ApplicationQueryCardinality;
pub(super) fn validate(definition: &WorthQueryPortableApplicationQueryParts) -> Result<(), Denial> {
    let Some(orbit) = definition.orbit() else {
        return Ok(());
    };
    if !orbit.query_identity().is_valid()
        || !orbit.slot_identity().is_valid()
        || orbit.query_type() != definition.query_type()
        || orbit.relation().trim().is_empty()
        || orbit.to() != orbit.from()
        || orbit.from() != definition.root_entity()
        || orbit.from() != definition.scope_entity()
        || orbit.output_name().trim().is_empty()
        || !definition.root_paths().is_empty()
        || !definition.predicates().is_empty()
        || !definition.ordering().is_empty()
        || definition.continuation().is_some()
        || definition.cardinality() != ApplicationQueryCardinality::Many
        || definition.lanes().live_enabled()
        || definition.live_cause().is_some()
    {
        return Err(Denial::InvalidOrbitShape);
    }
    Ok(())
}
