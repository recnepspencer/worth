use crate::error::{
    BridgeErrorContext, BridgePatchTargetCoordinate, BridgeRouteError, BridgeRouteErrorKind,
};
use crate::mapping::TruthDeltaSurfaceKind;

use super::{
    BridgeAuthoritativePatchLoweringCounters, BridgeCommittedPatchBody, BridgeCommittedPatchDigest,
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem,
    BridgeCommittedPatchSummary, BridgeCommittedRecordChange, BridgeEnvelopeCore,
    BridgePatchEnvelopeHeader, CanonicalPatchEnvelopeBody,
};

mod producer_authority;

use producer_authority::{
    committed_patch_digest_from_basis, digest_basis, validate_producer_metadata,
};

pub(super) fn construct_committed_patch_envelope(
    envelope_identity: BridgeCommittedPatchEnvelopeIdentity,
    patch_items: Vec<BridgeCommittedPatchItem>,
) -> Result<BridgeCommittedPatchEnvelope, BridgeRouteError> {
    construct_committed_patch_envelope_with_record_changes(
        envelope_identity,
        patch_items,
        Vec::new(),
        BridgeAuthoritativePatchLoweringCounters::default(),
    )
}

pub(super) fn construct_committed_patch_envelope_with_record_changes(
    envelope_identity: BridgeCommittedPatchEnvelopeIdentity,
    patch_items: Vec<BridgeCommittedPatchItem>,
    record_changes: Vec<BridgeCommittedRecordChange>,
    authoritative_lowering: BridgeAuthoritativePatchLoweringCounters,
) -> Result<BridgeCommittedPatchEnvelope, BridgeRouteError> {
    validate_producer_metadata(envelope_identity.producer_metadata())?;
    validate_identity(
        "commit identity",
        envelope_identity.commit_identity().as_str(),
    )?;
    validate_identity(
        "patch identity",
        envelope_identity.patch_identity().as_str(),
    )?;
    validate_identity(
        "snapshot identity",
        envelope_identity.snapshot_identity().as_str(),
    )?;
    validate_identity(
        "branch identity",
        envelope_identity.branch_identity().as_str(),
    )?;

    let patch_item_count = patch_items.len();
    let mut canonical_items = patch_items;
    canonical_items.sort_by(canonical_patch_item_order);
    canonical_items.dedup();
    let mut canonical_record_changes = record_changes;
    canonical_record_changes.sort_by_key(BridgeCommittedRecordChange::canonical_basis);
    canonical_record_changes.dedup();

    if canonical_items.is_empty() && canonical_record_changes.is_empty() {
        return Err(BridgeRouteError::new(
            BridgeRouteErrorKind::UnsupportedTruthPatchScope,
            "Committed patch envelope contained no canonical patch items to route.",
        ));
    }

    for (index, item) in canonical_items.iter().enumerate() {
        validate_patch_item(index, item)?;
    }

    let normalized_patch_item_count = canonical_items.len();
    let patch_summary = BridgeCommittedPatchSummary::new(
        patch_item_count,
        normalized_patch_item_count,
        canonical_record_changes.len(),
        authoritative_lowering,
    );
    let committed_patch_digest_basis = digest_basis(
        envelope_identity.producer_metadata(),
        envelope_identity.commit_identity(),
        envelope_identity.patch_identity(),
        envelope_identity.snapshot_identity(),
        envelope_identity.branch_identity(),
        &canonical_items,
        &canonical_record_changes,
        normalized_patch_item_count,
    );
    let digest = BridgeCommittedPatchDigest::admit_bridge_owned(committed_patch_digest_from_basis(
        &committed_patch_digest_basis,
    ));
    validate_identity("committed patch digest", digest.as_str())?;

    Ok(BridgeCommittedPatchEnvelope::from_core(
        BridgeEnvelopeCore::new(
            BridgePatchEnvelopeHeader::new(
                envelope_identity.producer_metadata().clone(),
                envelope_identity.commit_identity().clone(),
                envelope_identity.patch_identity().clone(),
                envelope_identity.snapshot_identity().clone(),
                envelope_identity.branch_identity().clone(),
            ),
            CanonicalPatchEnvelopeBody {
                patch_summary,
                patch_body: BridgeCommittedPatchBody::new(
                    canonical_items,
                    canonical_record_changes,
                ),
                digest,
            },
        ),
    ))
}

pub(super) fn canonical_patch_item_order(
    left: &BridgeCommittedPatchItem,
    right: &BridgeCommittedPatchItem,
) -> std::cmp::Ordering {
    left.entity_identity()
        .cmp(right.entity_identity())
        .then_with(|| left.aspect_locator().cmp(right.aspect_locator()))
        .then_with(|| left.surface_kind().cmp(&right.surface_kind()))
        .then_with(|| left.field_locator().cmp(&right.field_locator()))
        .then_with(|| left.canonical_basis().cmp(&right.canonical_basis()))
}

fn validate_patch_item(
    index: usize,
    item: &BridgeCommittedPatchItem,
) -> Result<(), BridgeRouteError> {
    validate_identity(
        &format!("patch item #{index} entity identity"),
        item.entity_identity(),
    )
    .map_err(|error| error.with_context(patch_context(item)))?;
    validate_identity(
        &format!("patch item #{index} target canonical basis"),
        &item.target_canonical_basis(),
    )
    .map_err(|error| error.with_context(patch_context(item)))?;

    match (item.surface_kind(), item.field_locator()) {
        (TruthDeltaSurfaceKind::EntityField, Some(_)) => {
            validate_authoritative_locator(item)?;
            validate_field_masks(item)
        }
        (TruthDeltaSurfaceKind::EntityField, None) => Err(BridgeRouteError::new(
            BridgeRouteErrorKind::UnsupportedTruthDeltaSurface,
            format!(
                "Committed patch item #{index} entity-field target lacked a foundational field locator."
            ),
        )
        .with_context(patch_context(item))),
        (_, Some(_)) => Err(BridgeRouteError::new(
            BridgeRouteErrorKind::UnsupportedTruthDeltaSurface,
            format!(
                "Committed patch item #{index} non-field target unexpectedly carried a foundational field locator."
            ),
        )
        .with_context(patch_context(item))),
        (_, None) => {
            validate_authoritative_locator(item)?;
            validate_whole_aspect_masks(item)
        }
    }?;
    validate_semantic_target(item)
}

fn validate_semantic_target(item: &BridgeCommittedPatchItem) -> Result<(), BridgeRouteError> {
    use worth_foundational::facade::{AspectBinding, AuthoritativeAspectChangeKind as Kind};
    let Some(change) = item.semantic_change() else {
        return Ok(());
    };
    if !matches!(
        (change.precision(), change.widening_cause()),
        (super::BridgeAspectChangePrecision::Exact, None)
            | (
                super::BridgeAspectChangePrecision::DeclaredWidening,
                Some(_)
            )
    ) {
        return Err(invalid_semantic_target(
            item,
            "semantic precision and widening evidence were inconsistent",
        ));
    }
    if change.aspect_key() != item.aspect_key() {
        return Err(invalid_semantic_target(
            item,
            "semantic aspect key did not match the authoritative target locator",
        ));
    }
    let field_binding = matches!(
        change.binding(),
        AspectBinding::EntityField { .. } | AspectBinding::RelationField { .. }
    );
    let valid = match change.kind() {
        Kind::FieldSet | Kind::FieldClear => {
            field_binding
                && item.surface_kind() == TruthDeltaSurfaceKind::EntityField
                && item.field_locator().map(|locator| locator.field_path()) == change.field_path()
        }
        Kind::WholeAspectSet | Kind::WholeAspectClear => {
            field_binding
                && item.surface_kind() == TruthDeltaSurfaceKind::AuthoritativeAspect
                && change.field_path().is_none()
        }
        Kind::RelationSourceEndpoint => {
            matches!(change.binding(), AspectBinding::RelationSourceEndpoint)
                && item.surface_kind() == TruthDeltaSurfaceKind::EntityRelationEndpoint
                && change.field_path().is_none()
        }
        Kind::RelationTargetEndpoint => {
            matches!(change.binding(), AspectBinding::RelationTargetEndpoint)
                && item.surface_kind() == TruthDeltaSurfaceKind::EntityRelationEndpoint
                && change.field_path().is_none()
        }
        Kind::LifecycleCreate | Kind::LifecycleDelete | Kind::LifecycleRetainForAudit => {
            matches!(change.binding(), AspectBinding::LifecycleTransition)
                && item.surface_kind() == TruthDeltaSurfaceKind::LifecycleTransition
                && change.field_path().is_none()
        }
        Kind::Opaque => {
            item.surface_kind() == TruthDeltaSurfaceKind::AuthoritativeAspect
                && change.field_path().is_none()
        }
        Kind::StructuralCreate
        | Kind::StructuralUpdate
        | Kind::StructuralDelete
        | Kind::StructuralRetainForAudit
        | Kind::StructuralMaterializationSuspended
        | Kind::StructuralRematerialized => {
            matches!(
                (change.binding(), item.surface_kind()),
                (
                    AspectBinding::StructuralRegion,
                    TruthDeltaSurfaceKind::EntityRegion
                ) | (
                    AspectBinding::StructuralPartition,
                    TruthDeltaSurfaceKind::EntityPartition
                ) | (
                    AspectBinding::StructuralFacet,
                    TruthDeltaSurfaceKind::EntityFacet
                )
            ) && change.field_path().is_none()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(invalid_semantic_target(
            item,
            "semantic change kind, binding, path, and target surface were inconsistent",
        ))
    }
}

fn invalid_semantic_target(
    item: &BridgeCommittedPatchItem,
    message: &'static str,
) -> BridgeRouteError {
    BridgeRouteError::new(
        BridgeRouteErrorKind::InvalidAuthoritativePatchSemantics,
        message,
    )
    .with_context(patch_context(item))
}

fn validate_authoritative_locator(item: &BridgeCommittedPatchItem) -> Result<(), BridgeRouteError> {
    if item.aspect_locator().authority()
        == worth_foundational::facade::LocatorAuthority::Authoritative
    {
        return Ok(());
    }

    Err(BridgeRouteError::new(
        BridgeRouteErrorKind::UnsupportedTruthDeltaSurface,
        "Committed patch targets must carry authoritative foundational aspect locators.",
    )
    .with_context(patch_context(item)))
}

fn validate_field_masks(item: &BridgeCommittedPatchItem) -> Result<(), BridgeRouteError> {
    let field_path = item
        .field_locator()
        .expect("field targets validated as carrying field locators")
        .field_path();
    let expected_path = std::slice::from_ref(field_path);
    let mutation_matches = item.mutation_mask().paths() == expected_path;
    let projection_matches = item.projection_mask().paths() == expected_path;
    if mutation_matches && projection_matches {
        return Ok(());
    }

    Err(BridgeRouteError::new(
        BridgeRouteErrorKind::UnsupportedTruthDeltaSurface,
        "Committed patch field targets must carry mutation and projection masks for the same foundational field path.",
    )
    .with_context(patch_context(item)))
}

fn validate_whole_aspect_masks(item: &BridgeCommittedPatchItem) -> Result<(), BridgeRouteError> {
    if item.mutation_mask().is_whole_aspect() && item.projection_mask().is_whole_aspect() {
        return Ok(());
    }

    Err(BridgeRouteError::new(
        BridgeRouteErrorKind::UnsupportedTruthDeltaSurface,
        "Committed patch non-field targets must carry whole-aspect mutation and projection masks.",
    )
    .with_context(patch_context(item)))
}

fn patch_context(item: &BridgeCommittedPatchItem) -> BridgeErrorContext {
    BridgeErrorContext::patch(BridgePatchTargetCoordinate::new(
        item.entity_identity(),
        item.target().clone(),
    ))
}

fn validate_identity(label: &str, value: &str) -> Result<(), BridgeRouteError> {
    if value.trim().is_empty() {
        return Err(BridgeRouteError::new(
            BridgeRouteErrorKind::UnsupportedTruthPatchScope,
            format!("Committed patch envelope {label} must be non-empty."),
        ));
    }

    Ok(())
}

#[cfg(test)]
#[path = "construction_tests.rs"]
mod construction_tests;
