use super::WorthQueryGraphReadStreamingPageBudget;
use crate::identity::hash_parts;
use crate::runtime::{
    WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessAdmissionPosture,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadStreamingPlan {
    digest: String,
    admission_digest: String,
    requirement_set_digest: String,
    page_budget: WorthQueryGraphReadStreamingPageBudget,
    canonical_result_basis_digest: String,
    replay_basis_digest: String,
}

impl WorthQueryGraphReadStreamingPlan {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn admission_digest(&self) -> &str {
        &self.admission_digest
    }

    pub fn requirement_set_digest(&self) -> &str {
        &self.requirement_set_digest
    }

    pub fn page_budget(&self) -> &WorthQueryGraphReadStreamingPageBudget {
        &self.page_budget
    }

    pub fn canonical_result_basis_digest(&self) -> &str {
        &self.canonical_result_basis_digest
    }

    pub fn replay_basis_digest(&self) -> &str {
        &self.replay_basis_digest
    }

    pub(crate) fn from_admission(admission: &WorthQueryGraphReadAccessAdmission) -> Option<Self> {
        if admission.posture() != &WorthQueryGraphReadAccessAdmissionPosture::AdmittedPagedStreaming
        {
            return None;
        }
        let page_budget = WorthQueryGraphReadStreamingPageBudget::frontier_default();
        let admission_digest = admission.digest().to_string();
        let requirement_set_digest = admission.requirement_set().digest().render_support_hex();
        let canonical_result_basis_digest = hash_parts(&[
            "worth_query_graph_read_streaming_canonical_result_basis_v1".to_string(),
            format!("requirements:{requirement_set_digest}"),
            format!(
                "selectivity:{}",
                admission
                    .requirement_set()
                    .selectivity_shape_digest()
                    .render_hex()
            ),
        ]);
        let replay_basis_digest = hash_parts(&[
            "worth_query_graph_read_streaming_replay_basis_v1".to_string(),
            format!("admission:{admission_digest}"),
            format!("requirements:{requirement_set_digest}"),
            format!("page_budget:{}", page_budget.digest()),
        ]);
        let digest = hash_parts(&[
            "worth_query_graph_read_streaming_plan_v2".to_string(),
            format!("admission:{admission_digest}"),
            format!("requirements:{requirement_set_digest}"),
            format!("page_budget:{}", page_budget.digest()),
            format!("canonical_result_basis:{canonical_result_basis_digest}"),
            format!("replay_basis:{replay_basis_digest}"),
        ]);
        Some(Self {
            digest,
            admission_digest,
            requirement_set_digest,
            page_budget,
            canonical_result_basis_digest,
            replay_basis_digest,
        })
    }
}
