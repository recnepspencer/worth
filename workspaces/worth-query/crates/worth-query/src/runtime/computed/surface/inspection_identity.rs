use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

pub(super) fn computed_definition_inspection_identity(
    name: &str,
    incremental_delivery: bool,
    upstream_live_views: &[String],
    upstream_derived_views: &[String],
    dependency_aspects: &[WorthQueryAspectTouch],
    produced_aspects: &[WorthQueryAspectTouch],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_computed_definition_inspection_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("name"), name)
        .field_shape(
            WorthQueryEvidenceTag::new("authority"),
            "derived_runtime_state",
        )
        .field_bool(
            WorthQueryEvidenceTag::new("incremental"),
            incremental_delivery,
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("live"),
            upstream_live_views.iter().map(String::as_str),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("derived"),
            upstream_derived_views.iter().map(String::as_str),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("dependencies"),
            dependency_aspects
                .iter()
                .map(WorthQueryAspectTouch::admitted_touch_digest_part),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("produces"),
            produced_aspects
                .iter()
                .map(WorthQueryAspectTouch::admitted_touch_digest_part),
        )
        .seal()
}

pub(super) fn computed_dependency_inspection_identity(
    name: &str,
    upstream_live_views: &[String],
    upstream_derived_views: &[String],
    dependency_aspects: &[WorthQueryAspectTouch],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_computed_dependency_inspection_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("name"), name)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("live"),
            upstream_live_views.iter().map(String::as_str),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("derived"),
            upstream_derived_views.iter().map(String::as_str),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("dependencies"),
            dependency_aspects
                .iter()
                .map(WorthQueryAspectTouch::admitted_touch_digest_part),
        )
        .seal()
}

pub(super) fn computed_produced_aspect_inspection_identity(
    name: &str,
    produced_aspects: &[WorthQueryAspectTouch],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_computed_produced_aspect_inspection_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("name"), name)
        .field_shape(
            WorthQueryEvidenceTag::new("authority"),
            "derived_runtime_state",
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("produces"),
            produced_aspects
                .iter()
                .map(WorthQueryAspectTouch::admitted_touch_digest_part),
        )
        .seal()
}

pub(super) fn computed_materialization_inspection_identity(
    name: &str,
    materialized_row_count: usize,
    rows: &[WorthQueryRetainedMaterializedRow],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let row_shapes: Vec<String> = rows
        .iter()
        .map(|row| row.terminal_digest_parts().join(";"))
        .collect();
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_computed_materialization_inspection_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("name"), name)
        .field_usize(WorthQueryEvidenceTag::new("rows"), materialized_row_count)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("materialized_rows"),
            row_shapes.iter().map(String::as_str),
        )
        .seal()
}

pub(super) fn computed_pending_patch_inspection_identity(
    name: &str,
    pending_patch_count: usize,
    pending_incremental_patch_count: usize,
    pending_refresh_fallback_count: usize,
    patch_identities: &[WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_computed_pending_patch_inspection_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("name"), name)
        .field_usize(WorthQueryEvidenceTag::new("pending"), pending_patch_count)
        .field_usize(
            WorthQueryEvidenceTag::new("incremental"),
            pending_incremental_patch_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("refresh"),
            pending_refresh_fallback_count,
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("patches"),
            patch_identities.iter(),
        )
        .seal()
}

pub(super) fn derived_patch_inspection_identity(
    view_name: &str,
    patch: &WorthQueryDerivedPatch,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let family = match patch.family() {
        WorthQueryDerivedPatchFamily::Incremental => "incremental",
        WorthQueryDerivedPatchFamily::RefreshFallback => "refresh_fallback",
    };
    let mut encoder =
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_computed_patch_inspection_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("view_name"), view_name)
            .field_shape(WorthQueryEvidenceTag::new("family"), family)
            .field_shape(
                WorthQueryEvidenceTag::new("authority_lane"),
                "derived_runtime_state",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("commit"),
                &patch.commit_identity().evidence_identity(),
            );
    if let Some(entity) = patch.entity_identity() {
        encoder = encoder.field_evidence_identity(
            WorthQueryEvidenceTag::new("entity"),
            &entity.evidence_identity(),
        );
    }
    if let Some(reason) = patch.reason() {
        encoder = encoder.field_shape(WorthQueryEvidenceTag::new("reason"), reason);
    }
    if !patch.aspect_touches().is_empty() {
        encoder = encoder.field_value_sequence(
            WorthQueryEvidenceTag::new("admitted_aspect_touches"),
            patch
                .aspect_touches()
                .iter()
                .map(WorthQueryAspectTouch::admitted_touch_digest_part),
        );
    }
    encoder.seal()
}
