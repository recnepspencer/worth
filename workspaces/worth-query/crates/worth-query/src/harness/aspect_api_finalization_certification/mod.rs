mod digests;
mod fixture;
mod rejections;
mod rows;
#[cfg(test)]
mod tests;

use crate::harness::certification::{digest_parts, CertificationMatrix};
use crate::runtime::WorthQueryAspectTouch;
use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};

pub const ASPECT_API_FINALIZATION_REQUIRED_CANONICAL_ROW_NAMES: &[&str] = &[
    "authoritative-insert-update-delete-surface",
    "typed-clear-narrows-by-touched-meaning",
    "preview-batch-lane-isolation",
    "mutation-surface-closeout-contract-sync",
];

pub const ASPECT_API_FINALIZATION_REQUIRED_REJECTION_ROW_NAMES: &[&str] = &[
    "unsupported-intent-family-fails-typed-and-early",
    "duplicate-clear-and-set-denied-before-routing",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AspectApiFinalizationPerturbationClass {
    AuthoritativeCrudSurface,
    TypedClearNarrowing,
    PreviewBatchIsolation,
    MutationSurfaceCloseoutSync,
    UnsupportedIntentFamilyDenied,
    DuplicateAspectAuthoringDenied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AspectApiFinalizationFailureClass {
    SupportDenied,
    AuthoringDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspectApiFinalizationCertificationBundle {
    pub mutation_surface_label: String,
    pub authority_lane_label: String,
    pub mutation_family_label: String,
    pub support_matrix_digest: String,
    pub mutation_surface_report_digest: String,
    pub closeout_digest: String,
    pub receipt_digest: String,
    pub state_digest: String,
    pub inspection_digest: String,
    pub touched_aspect_digest: String,
    pub affected_live_view_count: usize,
    pub affected_derived_view_count: usize,
    pub routed_patch_count: usize,
    pub materialized_row_count: usize,
    pub preview_residue_count: usize,
}

impl AspectApiFinalizationCertificationBundle {
    pub(super) fn has_required_outputs(&self) -> bool {
        !self.mutation_surface_label.is_empty()
            && !self.authority_lane_label.is_empty()
            && !self.mutation_family_label.is_empty()
            && !self.support_matrix_digest.is_empty()
            && !self.mutation_surface_report_digest.is_empty()
            && !self.closeout_digest.is_empty()
            && !self.receipt_digest.is_empty()
            && !self.state_digest.is_empty()
            && !self.inspection_digest.is_empty()
            && !self.touched_aspect_digest.is_empty()
    }

    pub(super) fn semantic_signature(&self) -> String {
        digest_parts(&[
            format!("surface:{}", self.mutation_surface_label),
            format!("lane:{}", self.authority_lane_label),
            format!("family:{}", self.mutation_family_label),
            format!("touched:{}", self.touched_aspect_digest),
            format!("live_count:{}", self.affected_live_view_count),
            format!("derived_count:{}", self.affected_derived_view_count),
            format!("patches:{}", self.routed_patch_count),
            format!("rows:{}", self.materialized_row_count),
            format!("preview_residue:{}", self.preview_residue_count),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspectApiFinalizationRejectionBundle {
    pub failure_class: AspectApiFinalizationFailureClass,
    pub failure_kind: String,
    pub failure_digest: String,
    pub support_matrix_digest: String,
    pub mutation_surface_report_digest: String,
    pub closeout_digest: String,
}

pub type AspectApiFinalizationCertificationMatrix = CertificationMatrix<
    AspectApiFinalizationPerturbationClass,
    AspectApiFinalizationCertificationBundle,
    AspectApiFinalizationRejectionBundle,
>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspectApiFinalizationCertificationArtifact {
    pub suite_name: &'static str,
    pub certification_bundle_digest: String,
    pub coverage_matrix_digest: String,
    pub matrix: AspectApiFinalizationCertificationMatrix,
}

impl AspectApiFinalizationCertificationMatrix {
    pub fn into_artifact(self) -> AspectApiFinalizationCertificationArtifact {
        AspectApiFinalizationCertificationArtifact {
            suite_name: self.suite_name,
            certification_bundle_digest: digest_parts(&digests::bundle_digest_parts(&self)),
            coverage_matrix_digest: digest_parts(&digests::coverage_digest_parts(&self)),
            matrix: self,
        }
    }
}

pub struct AspectApiFinalizationCertificationAdapter;

impl AspectApiFinalizationCertificationAdapter {
    pub fn public_aspect_api_finalization_artifact(
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> AspectApiFinalizationCertificationArtifact {
        Self::public_aspect_api_finalization_test(request_execution).into_artifact()
    }

    pub fn public_aspect_api_finalization_test(
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> AspectApiFinalizationCertificationMatrix {
        AspectApiFinalizationCertificationMatrix {
            suite_name: "Public Aspect API Finalization Test",
            rows: rows::canonical_rows(request_execution),
            rejection_rows: rows::rejection_rows(request_execution),
        }
    }
}

pub(super) fn identity_id_touch() -> WorthQueryAspectTouch {
    aspect_api_certification_touch("identity", "id")
}

pub(super) fn title_value_touch() -> WorthQueryAspectTouch {
    aspect_api_certification_touch("title", "value")
}

pub(super) fn description_value_touch() -> WorthQueryAspectTouch {
    aspect_api_certification_touch("description", "value")
}

pub(super) fn ui_batch_summary_touch() -> WorthQueryAspectTouch {
    aspect_api_certification_touch("ui", "batch_summary")
}

fn aspect_api_certification_touch(
    aspect_label: &'static str,
    field_label: &'static str,
) -> WorthQueryAspectTouch {
    let aspect_key =
        AspectKey::new(aspect_label).expect("aspect API certification aspect key should admit");
    let field_key =
        FieldKey::new(field_label).expect("aspect API certification field key should admit");
    let field_path = CanonicalFieldPath::new([field_key])
        .expect("aspect API certification field path should admit");
    WorthQueryAspectTouch::aspect_field_path(aspect_key, field_path)
}
