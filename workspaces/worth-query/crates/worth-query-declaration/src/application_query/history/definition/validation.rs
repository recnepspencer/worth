use std::collections::BTreeSet;

use super::super::ApplicationHistoryTarget;
use super::{
    ApplicationHistoryQueryDefinitionDenial as Denial,
    WorthQueryPortableApplicationHistoryQueryParts,
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
    parts
        .metadata_disclosure
        .validate_history_metadata(&parts.history_type)
        .map_err(|_| Denial::InvalidMetadataDisclosure)
}

fn name_is_valid(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
}
