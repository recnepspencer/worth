use crate::application_query::{
    ApplicationQueryDisclosurePosture, WorthQueryPortableApplicationQueryParts,
};

use super::{
    result_shape::{field_by_slot, relation_by_slot},
    ApplicationQueryDefinitionDenial,
};

pub(super) fn validate(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    let disclosure = definition.disclosure();
    let capability_is_complete = match disclosure.posture() {
        ApplicationQueryDisclosurePosture::Public
        | ApplicationQueryDisclosurePosture::InstalledPolicyRequired => {
            disclosure.capability_name().is_none() && disclosure.capability_type().is_none()
        }
        ApplicationQueryDisclosurePosture::Governed => {
            disclosure
                .capability_name()
                .is_some_and(|name| !name.trim().is_empty())
                && disclosure.capability_type().is_some()
        }
    };
    if disclosure.classification().trim().is_empty() || !capability_is_complete {
        return Err(ApplicationQueryDefinitionDenial::InvalidDisclosureContract);
    }
    for rule in disclosure.rules() {
        let selector = rule.selector();
        if !selector.has_exact_field_masks() {
            return Err(ApplicationQueryDefinitionDenial::DisclosureSelectorMismatch);
        }
        if selector.is_internal_computation() {
            continue;
        }
        let Some(expected) = selector.result_slot_key() else {
            return Err(ApplicationQueryDefinitionDenial::DisclosureSelectorMismatch);
        };
        let matches_field = field_by_slot(definition.result_shape(), selector.slot_type())
            .is_some_and(|field| field.slot_key() == expected);
        let matches_relation = relation_by_slot(definition.result_shape(), selector.slot_type())
            .is_some_and(|relation| relation.slot_key() == expected);
        let matches_orbit = definition
            .orbit()
            .is_some_and(|orbit| orbit.slot_key() == expected);
        if !matches_field && !matches_relation && !matches_orbit {
            return Err(ApplicationQueryDefinitionDenial::DisclosureSelectorMismatch);
        }
    }
    Ok(())
}
