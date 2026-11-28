use super::*;

pub(super) fn validate_registration_set(
    registrations: &[BridgeMappingRegistration],
) -> Result<(), BridgeBuildError> {
    for (index, left) in registrations.iter().enumerate() {
        for right in registrations.iter().skip(index + 1) {
            if left.mapping_id() == right.mapping_id() {
                return Err(BridgeBuildError::new(
                    BridgeBuildErrorKind::DuplicateMappingRegistration,
                    format!(
                        "Duplicate bridge mapping id `{}` detected across multiple registrations.",
                        left.mapping_id().as_str()
                    ),
                )
                .with_context(duplicate_mapping_context(left, right)));
            }

            if !left.truth_scope().overlaps(right.truth_scope()) {
                continue;
            }

            if left.semantic_duplicate_of(right) {
                return Err(BridgeBuildError::new(
                    BridgeBuildErrorKind::DuplicateMappingRegistration,
                    format!(
                        "Duplicate bridge mapping registration detected for truth scope between `{}` and `{}`.",
                        left.mapping_id().as_str(),
                        right.mapping_id().as_str()
                    ),
                )
                .with_context(duplicate_mapping_context(left, right)));
            }

            if left.truth_scope() == right.truth_scope() {
                continue;
            }

            if left.truth_scope().specificity_rank() == right.truth_scope().specificity_rank() {
                return Err(BridgeBuildError::new(
                    BridgeBuildErrorKind::AmbiguousMappingRegistration,
                    format!(
                        "Ambiguous bridge mapping registration overlap detected between `{}` and `{}`.",
                        left.mapping_id().as_str(),
                        right.mapping_id().as_str()
                    ),
                )
                .with_context(duplicate_mapping_context(left, right)));
            }
        }
    }

    Ok(())
}

pub(super) fn validate_registration_values(
    registrations: &[BridgeMappingRegistration],
) -> Result<(), BridgeBuildError> {
    for registration in registrations {
        validate_non_empty("mapping id", registration.mapping_id().as_str()).map_err(|error| {
            error.with_context(invalid_mapping_field_context(registration, "mapping id"))
        })?;
        validate_entity_selector(
            "entity selector",
            registration.truth_scope().entity_selector(),
        )
        .map_err(|error| {
            error.with_context(invalid_mapping_field_context(
                registration,
                "entity selector",
            ))
        })?;
        validate_aspect_selector(
            "aspect selector",
            registration.truth_scope().aspect_selector(),
        )
        .map_err(|error| {
            error.with_context(invalid_mapping_field_context(
                registration,
                "aspect selector",
            ))
        })?;
        validate_read_contract_scope(registration).map_err(|error| {
            error.with_context(invalid_mapping_field_context(
                registration,
                "snapshot read contract",
            ))
        })?;
        validate_target_selector(registration.truth_scope().target_selector()).map_err(
            |error| {
                error.with_context(invalid_mapping_field_context(
                    registration,
                    "target selector",
                ))
            },
        )?;
        validate_non_empty("signal scope", registration.signal_scope().as_str()).map_err(
            |error| error.with_context(invalid_mapping_field_context(registration, "signal scope")),
        )?;
    }

    Ok(())
}

fn validate_read_contract_scope(
    registration: &BridgeMappingRegistration,
) -> Result<(), BridgeBuildError> {
    match registration.truth_scope().aspect_selector() {
        AspectKeySelector::Exact(aspect_key)
            if aspect_key == registration.snapshot_read_contract().aspect_key() =>
        {
            Ok(())
        }
        AspectKeySelector::Exact(aspect_key) => Err(BridgeBuildError::new(
            BridgeBuildErrorKind::AmbiguousMappingRegistration,
            format!(
                "Bridge mapping `{}` declares read contract `{}` for truth aspect `{}`.",
                registration.mapping_id().as_str(),
                registration.snapshot_read_contract().aspect_key().as_str(),
                aspect_key.as_str()
            ),
        )),
        AspectKeySelector::Any => Err(BridgeBuildError::new(
            BridgeBuildErrorKind::AmbiguousMappingRegistration,
            format!(
                "Bridge mapping `{}` must declare an exact truth aspect for its snapshot read contract.",
                registration.mapping_id().as_str()
            ),
        )),
    }
}

fn duplicate_mapping_context(
    left: &BridgeMappingRegistration,
    right: &BridgeMappingRegistration,
) -> crate::error::BridgeErrorContext {
    crate::error::BridgeErrorContext::mapping_freeze(
        crate::error::BridgeMappingFreezeContext::for_mapping_pair(
            left.mapping_id().clone(),
            right.mapping_id().clone(),
        ),
    )
}

fn invalid_mapping_field_context(
    registration: &BridgeMappingRegistration,
    invalid_field: &'static str,
) -> crate::error::BridgeErrorContext {
    crate::error::BridgeErrorContext::mapping_freeze(
        crate::error::BridgeMappingFreezeContext::for_mapping(registration.mapping_id().clone())
            .with_invalid_field(invalid_field),
    )
}

fn validate_entity_selector(
    label: &str,
    selector: &MappingSelector,
) -> Result<(), BridgeBuildError> {
    match selector {
        MappingSelector::Any => Ok(()),
        MappingSelector::Exact(value) => validate_non_empty(label, value),
    }
}

fn validate_aspect_selector(
    label: &str,
    selector: &AspectKeySelector,
) -> Result<(), BridgeBuildError> {
    match selector {
        AspectKeySelector::Any => Ok(()),
        AspectKeySelector::Exact(value) => validate_non_empty(label, value.as_str()),
    }
}

fn validate_target_selector(selector: &TruthPatchTargetSelector) -> Result<(), BridgeBuildError> {
    match selector {
        TruthPatchTargetSelector::Any
        | TruthPatchTargetSelector::AuthoritativeAspect
        | TruthPatchTargetSelector::EntityRelationEndpoint
        | TruthPatchTargetSelector::EntityRegion
        | TruthPatchTargetSelector::EntityPartition
        | TruthPatchTargetSelector::EntityFacet
        | TruthPatchTargetSelector::LifecycleTransition => Ok(()),
        TruthPatchTargetSelector::EntityField(path) => {
            if path.fields().is_empty() {
                return Err(BridgeBuildError::new(
                    BridgeBuildErrorKind::AmbiguousMappingRegistration,
                    "Bridge mapping field target selector must carry a canonical field path.",
                ));
            }

            Ok(())
        }
    }
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), BridgeBuildError> {
    if value.trim().is_empty() {
        return Err(BridgeBuildError::new(
            BridgeBuildErrorKind::AmbiguousMappingRegistration,
            format!("Bridge mapping {label} must be non-empty."),
        ));
    }

    Ok(())
}

impl crate::mapping::registration::TruthPatchScope {
    pub(crate) fn matches_key(&self, key: &BridgeMappingLookupKey<'_>) -> bool {
        self.entity_selector().matches(key.entity_identity())
            && self.aspect_selector().matches(key.aspect_key())
            && self.target_selector().matches(key)
    }
}
