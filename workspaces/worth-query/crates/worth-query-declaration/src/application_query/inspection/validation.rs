use std::collections::BTreeSet;

use super::{
    ApplicationQueryInspectionDefinitionDenial as Denial, ApplicationQueryInspectionMetadataFacet,
    WorthQueryPortableApplicationQueryInspectionParts,
};
use crate::application_query::{
    ApplicationQueryDisclosurePosture, ApplicationQueryDisclosureSelector,
};

pub(super) fn validate(
    parts: &WorthQueryPortableApplicationQueryInspectionParts,
) -> Result<(), Denial> {
    if parts.semantic_version != 1 {
        return Err(Denial::UnsupportedSemanticVersion);
    }
    if !binding_is_valid(&parts.binding) {
        return Err(Denial::InvalidBindingReference);
    }
    let contract = &parts.metadata_disclosure;
    if contract.classification().trim().is_empty() {
        return Err(Denial::InvalidMetadataDisclosure);
    }
    match contract.posture() {
        ApplicationQueryDisclosurePosture::Public
            if contract.rules().is_empty()
                && contract.capability_name().is_none()
                && contract.capability_type().is_none() =>
        {
            return Ok(())
        }
        ApplicationQueryDisclosurePosture::Governed
            if contract.capability_name().is_some_and(name_is_valid)
                && contract
                    .capability_identity()
                    .is_some_and(|identity| identity.is_valid()) => {}
        _ => return Err(Denial::InvalidMetadataDisclosure),
    }
    if contract.rules().windows(2).any(|pair| pair[0] > pair[1]) {
        return Err(Denial::InvalidMetadataDisclosure);
    }
    let mut seen = BTreeSet::new();
    for rule in contract.rules() {
        let ApplicationQueryDisclosureSelector::InspectionMetadata(metadata) = rule.selector()
        else {
            return Err(Denial::InvalidMetadataDisclosure);
        };
        if metadata.binding() != &parts.binding
            || !rule.influence().permits(metadata.facet().influence())
            || !seen.insert(metadata.facet())
        {
            return Err(Denial::InvalidMetadataDisclosure);
        }
    }
    if seen.len() != ApplicationQueryInspectionMetadataFacet::ALL.len() {
        return Err(Denial::InvalidMetadataDisclosure);
    }
    Ok(())
}

pub(in crate::application_query) fn binding_is_valid(
    binding: &crate::application_query::ApplicationQueryBindingReference,
) -> bool {
    let p = binding.parts();
    [
        &p.binding,
        &p.input,
        &p.query,
        &p.parameters,
        &p.result,
        &p.scope,
        &p.scope_value,
    ]
    .iter()
    .all(|identity| identity.is_valid())
        && binding
            .canonical_fields()
            .iter()
            .all(|(_, value)| name_is_valid(value))
}

fn name_is_valid(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
}
