use super::{
    ApplicationManagedIndexDefinitionDenial as Denial, WorthQueryPortableManagedIndexParts,
};

pub(super) fn validate_definition(
    parts: &WorthQueryPortableManagedIndexParts,
) -> Result<(), Denial> {
    if parts.semantic_version != 1 {
        return Err(Denial::UnsupportedVersion {
            observed: parts.semantic_version,
        });
    }
    if [&parts.name, &parts.provider, &parts.entity]
        .into_iter()
        .any(|name| !valid_text(name))
        || parts.provider_version == 0
        || parts
            .source
            .binding
            .canonical_fields()
            .iter()
            .any(|(_, value)| !valid_text(value))
    {
        return Err(Denial::InvalidIdentifier);
    }
    if !parts.probe.has_supported_profile() {
        return Err(Denial::UnsupportedProbeProfile);
    }
    Ok(())
}

fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
}
