use crate::harness::certification::{digest_parts, CanonicalCertificationRow, CertificationMatrix};

use super::axes::{MilestoneNineFiveFailureClass, MilestoneNineFivePerturbationClass};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneNineFiveHostileLaneBundle {
    pub composition_axis: String,
    pub view_axis: String,
    pub projection_axis: String,
    pub reuse_axis: String,
    pub bootstrap_axis: String,
    pub canonical_query_digest: String,
    pub canonical_result_shape_digest: String,
    pub composition_authority_digest: String,
    pub composition_support_digest: String,
    pub view_shape_digest: String,
    pub view_plan_digest: String,
    pub view_support_digest: String,
    pub projection_contract_digest: String,
    pub projection_support_digest: String,
    pub projection_oracle_digest: String,
    pub saved_query_digest: String,
    pub reuse_matrix_digest: String,
    pub temporal_async_surface_posture: String,
    pub bootstrap_contract_digest: String,
    pub bootstrap_support_digest: String,
    pub application_support_report_digest: String,
}

impl MilestoneNineFiveHostileLaneBundle {
    pub(super) fn semantic_signature(&self) -> String {
        digest_parts(&[
            format!("query:{}", self.canonical_query_digest),
            format!("shape:{}", self.canonical_result_shape_digest),
            format!("view_shape:{}", self.view_shape_digest),
            format!("view_plan:{}", self.view_plan_digest),
            format!("projection:{}", self.projection_contract_digest),
            format!("projection_support:{}", self.projection_support_digest),
            format!("projection_oracle:{}", self.projection_oracle_digest),
            format!("saved:{}", self.saved_query_digest),
            format!("reuse:{}", self.reuse_matrix_digest),
            format!("temporal_async:{}", self.temporal_async_surface_posture),
            format!("bootstrap:{}", self.bootstrap_contract_digest),
            format!("bootstrap_support:{}", self.bootstrap_support_digest),
            format!("support_report:{}", self.application_support_report_digest),
        ])
    }

    pub(super) fn artifact_signature(&self) -> String {
        digest_parts(&[
            format!("composition_axis:{}", self.composition_axis),
            format!(
                "composition_authority:{}",
                self.composition_authority_digest
            ),
            format!("composition_support:{}", self.composition_support_digest),
            format!("query:{}", self.canonical_query_digest),
            format!("shape:{}", self.canonical_result_shape_digest),
            format!("view_axis:{}", self.view_axis),
            format!("view_shape:{}", self.view_shape_digest),
            format!("view_plan:{}", self.view_plan_digest),
            format!("view_support:{}", self.view_support_digest),
            format!("projection_axis:{}", self.projection_axis),
            format!("projection:{}", self.projection_contract_digest),
            format!("projection_support:{}", self.projection_support_digest),
            format!("projection_oracle:{}", self.projection_oracle_digest),
            format!("reuse_axis:{}", self.reuse_axis),
            format!("saved:{}", self.saved_query_digest),
            format!("reuse:{}", self.reuse_matrix_digest),
            format!("temporal_async:{}", self.temporal_async_surface_posture),
            format!("bootstrap_axis:{}", self.bootstrap_axis),
            format!("bootstrap:{}", self.bootstrap_contract_digest),
            format!("bootstrap_support:{}", self.bootstrap_support_digest),
            format!("support_report:{}", self.application_support_report_digest),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneNineFiveHostileRejectionBundle {
    pub failure_class: MilestoneNineFiveFailureClass,
    pub failure_kind: String,
    pub failure_digest: String,
    pub reuse_matrix_digest: String,
    pub temporal_async_surface_posture: String,
    pub counter_snapshot: String,
}

pub type MilestoneNineFiveHostileRow = CanonicalCertificationRow<
    MilestoneNineFivePerturbationClass,
    MilestoneNineFiveHostileLaneBundle,
>;

pub type MilestoneNineFiveHostileMatrix = CertificationMatrix<
    MilestoneNineFivePerturbationClass,
    MilestoneNineFiveHostileLaneBundle,
    MilestoneNineFiveHostileRejectionBundle,
>;
