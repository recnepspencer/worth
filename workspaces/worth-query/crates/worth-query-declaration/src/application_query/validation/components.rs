//! Whole cohorts must not acquire filtering, ordering or paging semantics.
use super::{ApplicationQueryDefinitionDenial as Denial, WorthQueryPortableApplicationQueryParts};
use crate::application_query::{ApplicationQueryCardinality, ApplicationQueryComponentSelection};
pub(super) fn validate(definition: &WorthQueryPortableApplicationQueryParts) -> Result<(), Denial> {
    let Some(selection) = definition.component_selection() else {
        return Ok(());
    };
    if !selection.query_identity().is_valid()
        || !selection.slot_identity().is_valid()
        || selection.query_type() != definition.query_type()
        || selection.set().trim().is_empty()
        || selection.output_name().trim().is_empty()
        || selection.member() != definition.root_entity()
        || (selection.selection() == ApplicationQueryComponentSelection::OfScopeMember
            && selection.member() != definition.scope_entity())
        || definition.orbit().is_some()
        || definition.root_paths().is_empty()
        || !definition.predicates().is_empty()
        || !definition.ordering().is_empty()
        || definition.continuation().is_some()
        || definition.cardinality() != ApplicationQueryCardinality::Many
        || definition.lanes().live_enabled()
        || definition.live_cause().is_some()
    {
        return Err(Denial::InvalidComponentShape);
    }
    Ok(())
}
