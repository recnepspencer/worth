use super::inspection_identity::{
    computed_definition_inspection_identity, computed_dependency_inspection_identity,
    computed_materialization_inspection_identity, computed_pending_patch_inspection_identity,
    computed_produced_aspect_inspection_identity, derived_patch_inspection_identity,
};
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryComputedInspectionEvidence {
    name: String,
    authority_lane: WorthQueryAuthorityLane,
    upstream_live_views: Vec<String>,
    upstream_derived_views: Vec<String>,
    dependency_aspects: Vec<WorthQueryAspectTouch>,
    produced_aspects: Vec<WorthQueryAspectTouch>,
    incremental_delivery: bool,
    materialized_row_count: usize,
    pending_patch_count: usize,
    pending_incremental_patch_count: usize,
    pending_refresh_fallback_count: usize,
    declaration_digest: String,
    dependency_digest: String,
    produced_aspect_digest: String,
    materialization_digest: String,
    pending_patch_digest: String,
    inspection_digest: String,
}

impl WorthQueryComputedInspectionEvidence {
    pub(in crate::runtime) fn from_runtime(view: &WorthQueryDerivedViewRuntime) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let upstream_live_views = view.declaration.upstream_live_views().to_vec();
        let upstream_derived_views = view.declaration.upstream_derived_views().to_vec();
        let dependency_aspects = view.declaration.dependency_aspect_touches().to_vec();
        let produced_aspects = view.declaration.produced_aspect_touches().to_vec();
        let incremental_delivery = view.declaration.incremental();
        let materialized_row_count = view.materialization.retained_rows().len();
        let pending_patch_count = view.patches.len();
        let pending_incremental_patch_count = view
            .patches
            .iter()
            .filter(|patch| !patch.is_refresh_fallback())
            .count();
        let pending_refresh_fallback_count = view
            .patches
            .iter()
            .filter(|patch| patch.is_refresh_fallback())
            .count();
        let declaration_identity = computed_definition_inspection_identity(
            view.declaration.name(),
            incremental_delivery,
            &upstream_live_views,
            &upstream_derived_views,
            &dependency_aspects,
            &produced_aspects,
        );
        let declaration_digest = declaration_identity.reporting_projection().to_string();
        let dependency_identity = computed_dependency_inspection_identity(
            view.declaration.name(),
            &upstream_live_views,
            &upstream_derived_views,
            &dependency_aspects,
        );
        let dependency_digest = dependency_identity.reporting_projection().to_string();
        let produced_aspect_identity = computed_produced_aspect_inspection_identity(
            view.declaration.name(),
            &produced_aspects,
        );
        let produced_aspect_digest = produced_aspect_identity.reporting_projection().to_string();
        let materialization_identity = computed_materialization_inspection_identity(
            view.declaration.name(),
            materialized_row_count,
            view.materialization.retained_rows(),
        );
        let materialization_digest = materialization_identity.reporting_projection().to_string();
        let patch_identities: Vec<WorthQueryEvidenceIdentity> = view
            .patches
            .iter()
            .map(|patch| derived_patch_inspection_identity(view.declaration.name(), patch))
            .collect();
        let pending_patch_identity = computed_pending_patch_inspection_identity(
            view.declaration.name(),
            pending_patch_count,
            pending_incremental_patch_count,
            pending_refresh_fallback_count,
            &patch_identities,
        );
        let pending_patch_digest = pending_patch_identity.reporting_projection().to_string();
        let inspection_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "worth_query_computed_inspection_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("declaration"),
                    &declaration_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("dependency"),
                    &dependency_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("produced_aspects"),
                    &produced_aspect_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("materialization"),
                    &materialization_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("pending_patches"),
                    &pending_patch_identity,
                )
                .seal()?;
        let inspection_digest = inspection_identity.reporting_projection().to_string();
        Ok(Self {
            name: view.declaration.name().to_string(),
            authority_lane: WorthQueryAuthorityLane::DerivedRuntimeState,
            upstream_live_views,
            upstream_derived_views,
            dependency_aspects,
            produced_aspects,
            incremental_delivery,
            materialized_row_count,
            pending_patch_count,
            pending_incremental_patch_count,
            pending_refresh_fallback_count,
            declaration_digest,
            dependency_digest,
            produced_aspect_digest,
            materialization_digest,
            pending_patch_digest,
            inspection_digest,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn upstream_live_views(&self) -> &[String] {
        &self.upstream_live_views
    }

    pub fn upstream_derived_views(&self) -> &[String] {
        &self.upstream_derived_views
    }

    pub fn dependency_aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.dependency_aspects
    }

    pub fn produced_aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.produced_aspects
    }

    pub fn incremental_delivery(&self) -> bool {
        self.incremental_delivery
    }

    pub fn materialized_row_count(&self) -> usize {
        self.materialized_row_count
    }

    pub fn pending_patch_count(&self) -> usize {
        self.pending_patch_count
    }

    pub fn pending_incremental_patch_count(&self) -> usize {
        self.pending_incremental_patch_count
    }

    pub fn pending_refresh_fallback_count(&self) -> usize {
        self.pending_refresh_fallback_count
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn dependency_digest(&self) -> &str {
        &self.dependency_digest
    }

    pub fn dependency_identity(&self) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_computed_dependency_inspection_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("name"), &self.name)
            .field_value_sequence(
                WorthQueryEvidenceTag::new("live"),
                self.upstream_live_views.iter().map(String::as_str),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("derived"),
                self.upstream_derived_views.iter().map(String::as_str),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("dependencies"),
                self.dependency_aspects
                    .iter()
                    .map(|touch| touch.admitted_touch_digest_part()),
            )
            .seal()?)
    }

    pub fn produced_aspect_digest(&self) -> &str {
        &self.produced_aspect_digest
    }

    pub fn materialization_digest(&self) -> &str {
        &self.materialization_digest
    }

    pub fn materialization_identity(&self) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_computed_materialization_inspection_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("name"), &self.name)
            .field_usize(
                WorthQueryEvidenceTag::new("rows"),
                self.materialized_row_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_patches"),
                self.pending_patch_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_incremental_patches"),
                self.pending_incremental_patch_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_refresh_fallbacks"),
                self.pending_refresh_fallback_count,
            )
            .seal()?)
    }

    pub fn pending_patch_digest(&self) -> &str {
        &self.pending_patch_digest
    }

    pub fn inspection_digest(&self) -> &str {
        &self.inspection_digest
    }
}
