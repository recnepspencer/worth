use crate::harness::certification::CanonicalCertificationRow;
use crate::harness::certification::CertificationMatrix;
use crate::harness::certification::RejectionCertificationRow;
use crate::harness::milestone_nine_certification::classifications::MilestoneNineFailureClass;
use crate::harness::milestone_nine_certification::classifications::MilestoneNinePerturbationClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneNineCertificationBundle {
    pub canonical_query_digest: String,
    pub policy_digest: String,
    pub result_digest: String,
    pub tenant_truth_basis_digest: String,
    pub tenant_schema_basis_digest: String,
    pub branch_access_digest: String,
    pub schema_variant_digest: String,
    pub execution_mode: String,
    pub admission_disposition: String,
    pub policy_cost_posture: String,
    pub policy_work_budget_digest: String,
    pub authorized_projection_digest: String,
    pub narrowed_result_shape_digest: String,
    pub relationship_proof_digest: String,
    pub validation_report_digest: String,
    pub policy_plan_digest: String,
    pub policy_execution_seam_digest: String,
    pub delivery_digest: String,
    pub employee_fixture_digest: String,
    pub policy_scale_counter_slope_digest: String,
    pub live_drift_evidence_digest: String,
    pub delivery_width_class_digest: String,
    pub composition_policy_parity_digest: String,
    pub view_shape_policy_parity_digest: String,
    pub placeholder_denial_digest: String,
    pub counter_snapshot_digest: String,
    pub support_profile_digest: String,
}

impl MilestoneNineCertificationBundle {
    pub(in crate::harness::milestone_nine_certification) fn has_required_outputs(&self) -> bool {
        !self.canonical_query_digest.is_empty()
            && !self.policy_digest.is_empty()
            && !self.result_digest.is_empty()
            && !self.tenant_truth_basis_digest.is_empty()
            && !self.tenant_schema_basis_digest.is_empty()
            && !self.branch_access_digest.is_empty()
            && !self.schema_variant_digest.is_empty()
            && !self.policy_cost_posture.is_empty()
            && !self.policy_work_budget_digest.is_empty()
            && !self.authorized_projection_digest.is_empty()
            && !self.narrowed_result_shape_digest.is_empty()
            && !self.relationship_proof_digest.is_empty()
            && !self.validation_report_digest.is_empty()
            && !self.policy_plan_digest.is_empty()
            && !self.policy_execution_seam_digest.is_empty()
            && !self.delivery_digest.is_empty()
            && !self.employee_fixture_digest.is_empty()
            && !self.policy_scale_counter_slope_digest.is_empty()
            && !self.live_drift_evidence_digest.is_empty()
            && !self.delivery_width_class_digest.is_empty()
            && !self.composition_policy_parity_digest.is_empty()
            && !self.view_shape_policy_parity_digest.is_empty()
            && !self.placeholder_denial_digest.is_empty()
            && !self.counter_snapshot_digest.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneNineRejectionBundle {
    pub failure_class: MilestoneNineFailureClass,
    pub failure_digest: String,
    pub counter_snapshot_digest: String,
}

pub type MilestoneNineCertificationRow =
    CanonicalCertificationRow<MilestoneNinePerturbationClass, MilestoneNineCertificationBundle>;

pub type MilestoneNineRejectionRow = RejectionCertificationRow<
    MilestoneNinePerturbationClass,
    MilestoneNineCertificationBundle,
    MilestoneNineRejectionBundle,
>;

pub type MilestoneNineCertificationMatrix = CertificationMatrix<
    MilestoneNinePerturbationClass,
    MilestoneNineCertificationBundle,
    MilestoneNineRejectionBundle,
>;
