use worth_relational::facade::identity::EntityId;

use super::super::effect_program::WorthQueryApplicationRealizedEffect;
use super::super::WorthQueryApplicationObservedFact;

pub(super) fn is_output_currentness_fact(fact: &WorthQueryApplicationObservedFact) -> bool {
    matches!(
        fact,
        WorthQueryApplicationObservedFact::SourceEntity { .. }
            | WorthQueryApplicationObservedFact::SourceAspectRevision { .. }
            | WorthQueryApplicationObservedFact::SourceFieldRevision { .. }
            | WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
            | WorthQueryApplicationObservedFact::Entity { .. }
            | WorthQueryApplicationObservedFact::Field { .. }
            | WorthQueryApplicationObservedFact::AbsentField { .. }
            | WorthQueryApplicationObservedFact::IndexedEntitySelection { .. }
    )
}

pub(super) fn normalized_output_facts(
    facts: &[WorthQueryApplicationObservedFact],
    effects: &[WorthQueryApplicationRealizedEffect],
) -> Vec<WorthQueryApplicationObservedFact> {
    facts
        .iter()
        .filter_map(|fact| match fact {
            WorthQueryApplicationObservedFact::Field {
                entity_id,
                kind,
                locator,
                value,
            } => Some(
                match replacement_field_value(*entity_id, locator, effects) {
                    Some(Some(replacement)) => WorthQueryApplicationObservedFact::Field {
                        entity_id: *entity_id,
                        kind: *kind,
                        locator: locator.clone(),
                        value: replacement.clone(),
                    },
                    Some(None) => WorthQueryApplicationObservedFact::AbsentField {
                        entity_id: *entity_id,
                        kind: *kind,
                        locator: locator.clone(),
                    },
                    None => WorthQueryApplicationObservedFact::Field {
                        entity_id: *entity_id,
                        kind: *kind,
                        locator: locator.clone(),
                        value: value.clone(),
                    },
                },
            ),
            WorthQueryApplicationObservedFact::AbsentField {
                entity_id,
                kind,
                locator,
            } => Some(
                match replacement_field_value(*entity_id, locator, effects) {
                    Some(Some(replacement)) => WorthQueryApplicationObservedFact::Field {
                        entity_id: *entity_id,
                        kind: *kind,
                        locator: locator.clone(),
                        value: replacement.clone(),
                    },
                    _ => fact.clone(),
                },
            ),
            _ => Some(fact.clone()),
        })
        .collect()
}

fn replacement_field_value<'effect>(
    entity_id: EntityId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    effects: &'effect [WorthQueryApplicationRealizedEffect],
) -> Option<Option<&'effect worth_foundational::facade::AspectValue>> {
    effects.iter().find_map(|effect| match effect {
        WorthQueryApplicationRealizedEffect::UpdateEntity {
            entity_id: candidate,
            fields,
            ..
        } if *candidate == entity_id => fields.get(locator).map(Some),
        WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
            entity_id: candidate,
            fields,
            ..
        } if *candidate == entity_id => fields.get(locator).map(|write| write.value.as_ref()),
        _ => None,
    })
}
