use std::collections::BTreeSet;

use super::super::{ApplicationHistoryMetadataFacet, ApplicationHistoryTarget};
use super::{
    ApplicationHistoryQueryDefinitionDenial as Denial,
    WorthQueryPortableApplicationHistoryQueryParts,
};
use crate::application_query::{
    ApplicationQueryDisclosurePosture, ApplicationQueryDisclosureSelector,
};

pub(super) fn validate(
    parts: &WorthQueryPortableApplicationHistoryQueryParts,
) -> Result<(), Denial> {
    if !name_is_valid(&parts.name)
        || !parts.history_type.is_valid()
        || parts
            .subject_read
            .canonical_fields()
            .iter()
            .any(|(_, value)| value.is_empty())
    {
        return Err(Denial::InvalidIdentity);
    }
    match &parts.target {
        ApplicationHistoryTarget::Entity { entity } if !name_is_valid(entity) => {
            return Err(Denial::InvalidTarget)
        }
        ApplicationHistoryTarget::Relation {
            query,
            slot,
            relation,
            from,
            to,
            output_name,
            ..
        } if !query.is_valid()
            || !slot.is_valid()
            || [relation, from, to, output_name]
                .into_iter()
                .any(|value| !name_is_valid(value)) =>
        {
            return Err(Denial::InvalidTarget);
        }
        _ => {}
    }
    if parts.aspects.is_empty() {
        return Err(Denial::EmptyAspectSelection);
    }
    if parts.aspects.windows(2).any(|pair| pair[0] > pair[1]) {
        return Err(Denial::NonCanonicalAspectSelection);
    }
    let mut names = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for aspect in &parts.aspects {
        if !name_is_valid(&aspect.aspect) || aspect.revision.0 == 0 {
            return Err(Denial::InvalidAspect);
        }
        if !names.insert(&aspect.aspect) || !identities.insert(aspect.identity) {
            return Err(Denial::DuplicateAspect);
        }
    }
    validate_metadata(parts)
}

fn validate_metadata(parts: &WorthQueryPortableApplicationHistoryQueryParts) -> Result<(), Denial> {
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
    let mut seen = BTreeSet::new();
    for rule in contract.rules() {
        let ApplicationQueryDisclosureSelector::HistoryMetadata(metadata) = rule.selector() else {
            return Err(Denial::InvalidMetadataDisclosure);
        };
        if metadata.history_identity() != parts.history_type
            || !rule.influence().permits(metadata.facet().influence())
            || !seen.insert(metadata.facet())
        {
            return Err(Denial::InvalidMetadataDisclosure);
        }
    }
    if seen.len() != ApplicationHistoryMetadataFacet::ALL.len() {
        return Err(Denial::InvalidMetadataDisclosure);
    }
    Ok(())
}

fn name_is_valid(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
}
