use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneEightCertificationBundle {
    pub query_digest: String,
    pub plan_digest: String,
    pub result_shape_digest: String,
    pub delivery_digest: String,
    pub counter_snapshot_digest: String,
    pub artifact_binding_matrix_digest: String,
    pub support_profile_digest: String,
    pub identity_consumption_digest: String,
    pub inspector_identity_digest: String,
    pub inspector_identity_classification: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneEightRejectionBundle {
    pub failure_class: MilestoneEightFailureClass,
    pub failure_digest: String,
    pub counter_snapshot_digest: String,
}

pub type MilestoneEightCertificationRow =
    CanonicalCertificationRow<MilestoneEightPerturbationClass, MilestoneEightCertificationBundle>;
pub type MilestoneEightRejectionRow = RejectionCertificationRow<
    MilestoneEightPerturbationClass,
    MilestoneEightCertificationBundle,
    MilestoneEightRejectionBundle,
>;
pub type MilestoneEightCertificationMatrix = CertificationMatrix<
    MilestoneEightPerturbationClass,
    MilestoneEightCertificationBundle,
    MilestoneEightRejectionBundle,
>;
