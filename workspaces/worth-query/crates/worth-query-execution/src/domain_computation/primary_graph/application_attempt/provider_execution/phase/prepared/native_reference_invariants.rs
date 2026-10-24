//! Native output references incur endpoint invariant duties without application write authority.
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationRealizedEffect as Effect,
    schema_layout::WorthQueryPrimaryGraphLayout,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_schema::ApplicationInvariantScopeTarget;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationInvariantCatalog, WorthQueryInvariantExecutionContract,
};
use worth_relational::facade::transactions::EntityReference;

pub(super) fn derive(
    layout: &WorthQueryPrimaryGraphLayout,
    catalog: &WorthQueryInstalledApplicationInvariantCatalog,
    effects: &[Effect],
) -> Result<WorthQueryInvariantExecutionContract, &'static str> {
    let entities = referenced_entities(layout, effects)?;
    if entities.is_empty() {
        return Ok(WorthQueryInvariantExecutionContract::NotRequired);
    }
    let descriptors = catalog
        .descriptors()
        .filter(|descriptor| applies_to_references(descriptor.applicability(), &entities))
        .cloned()
        .collect::<Vec<_>>();
    WorthQueryInvariantExecutionContract::for_installed_application_invariants(&descriptors)
}

fn applies_to_references(
    applicability: &[ApplicationInvariantScopeTarget],
    entities: &BTreeSet<&str>,
) -> bool {
    applicability.iter().any(|target| match target {
        ApplicationInvariantScopeTarget::Entity(entity) => entities.contains(entity.as_str()),
        ApplicationInvariantScopeTarget::Relation(_) => false,
    })
}

fn referenced_entities<'a>(
    layout: &WorthQueryPrimaryGraphLayout,
    effects: &'a [Effect],
) -> Result<BTreeSet<&'a str>, &'static str> {
    let output = &layout.workflow().operation_output;
    let mut entries = BTreeMap::new();
    for effect in effects {
        if let Effect::CreateEntity {
            kind, key, fields, ..
        } = effect
            && *kind == output.entity_kind
            && entries.insert(key.as_str(), fields).is_some()
        {
            return Err("native output reference has ambiguous output entries");
        }
    }
    let mut entities = BTreeSet::new();
    for effect in effects {
        let Effect::CreateRelation { kind, from, .. } = effect else {
            continue;
        };
        if *kind != output.target {
            continue;
        }
        let EntityReference::Created(entry) = from else {
            return Err("native output reference has no same-candidate output entry");
        };
        if entry.kind_id != output.entity_kind {
            return Err("native output reference has a foreign output entry kind");
        }
        let fields = entry
            .client_key
            .as_raw_str()
            .and_then(|key| entries.get(key))
            .ok_or("native output reference has no output entry")?;
        let Some(AspectValue::String(InternedString::Raw(entity))) = fields.get(&output.entity)
        else {
            return Err("native output reference has no declared entity kind");
        };
        if layout.entity_kind(entity).is_none() {
            return Err("native output reference names an uninstalled entity kind");
        }
        entities.insert(entity.as_str());
    }
    Ok(entities)
}

#[cfg(test)]
mod tests;
