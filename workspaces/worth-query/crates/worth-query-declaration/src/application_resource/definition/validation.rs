use std::collections::BTreeSet;

use crate::application_query::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryDisclosurePosture,
    ApplicationQueryDisclosureSelector,
};
use crate::application_resource::{ApplicationResourceAction, ApplicationResourceDisclosureFacet};

use super::{
    ApplicationResourceDefinitionDenial as Denial, WorthQueryPortableApplicationResourceParts,
};

pub(super) fn validate(parts: &WorthQueryPortableApplicationResourceParts) -> Result<(), Denial> {
    if !name_is_valid(&parts.name)
        || !parts.resource_type.is_valid()
        || !parts.provider_role.is_valid()
    {
        return Err(Denial::InvalidIdentity);
    }
    if parts
        .access
        .canonical_fields()
        .iter()
        .any(|(_, value)| value.is_empty())
    {
        return Err(Denial::InvalidAccess);
    }
    if [&parts.input, &parts.result, &parts.failure]
        .into_iter()
        .any(|value| !value.has_supported_profile())
    {
        return Err(Denial::UnsupportedValueProfile);
    }
    if !parts.profile.is_valid() {
        return Err(Denial::InvalidRequestResponseProfile);
    }
    if !parts.limits.is_valid() {
        return Err(Denial::InvalidLimits);
    }
    for action in ApplicationResourceAction::ALL {
        if let ApplicationQueryAuthorizationRequirement::Ability {
            ability,
            scope_entity,
        } = parts.authorization.requirement(action)
        {
            if !name_is_valid(ability) || scope_entity != &parts.access.parts().scope_entity {
                return Err(Denial::InvalidAuthorization);
            }
        }
    }
    validate_disclosure(parts)
}

fn validate_disclosure(parts: &WorthQueryPortableApplicationResourceParts) -> Result<(), Denial> {
    let contract = &parts.disclosure;
    if contract.classification().trim().is_empty() {
        return Err(Denial::InvalidDisclosure);
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
        _ => return Err(Denial::InvalidDisclosure),
    }
    let mut seen = BTreeSet::new();
    for rule in contract.rules() {
        let ApplicationQueryDisclosureSelector::Resource(meaning) = rule.selector() else {
            return Err(Denial::InvalidDisclosure);
        };
        if meaning.resource_identity() != parts.resource_type
            || meaning
                .facet()
                .required_influences()
                .iter()
                .any(|influence| !rule.influence().permits(*influence))
            || !seen.insert(meaning.facet())
        {
            return Err(Denial::InvalidDisclosure);
        }
    }
    if seen.len() != ApplicationResourceDisclosureFacet::ALL.len() {
        return Err(Denial::InvalidDisclosure);
    }
    Ok(())
}

fn name_is_valid(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
}
