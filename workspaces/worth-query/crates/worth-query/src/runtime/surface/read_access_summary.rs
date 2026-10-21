use crate::identity::hash_parts;
use crate::runtime::{
    WorthQueryAdmittedGraphReadAccessPlan, WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadAccessPlanConsumption,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessReceiptSummary {
    digest: String,
    read_graph_digest: String,
    admitted_plan_digest: String,
    admission_digest: String,
    plan_consumption_digest: String,
    authority_receipt_digest: String,
    admission_posture: WorthQueryGraphReadAccessAdmissionPosture,
    execution_strategy: String,
    requirement_set_digest: String,
    graph_index_inventory_match_report_digest: String,
}

impl WorthQueryGraphReadAccessReceiptSummary {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn read_graph_digest(&self) -> &str {
        &self.read_graph_digest
    }

    pub fn admitted_plan_digest(&self) -> &str {
        &self.admitted_plan_digest
    }

    pub fn plan_digest(&self) -> &str {
        &self.admitted_plan_digest
    }

    pub fn admission_digest(&self) -> &str {
        &self.admission_digest
    }

    pub fn plan_consumption_digest(&self) -> &str {
        &self.plan_consumption_digest
    }

    pub fn authority_receipt_digest(&self) -> &str {
        &self.authority_receipt_digest
    }

    pub fn admission_posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        &self.admission_posture
    }

    pub fn execution_strategy(&self) -> &str {
        &self.execution_strategy
    }

    pub fn requirement_set_digest(&self) -> &str {
        &self.requirement_set_digest
    }

    pub fn graph_index_inventory_match_report_digest(&self) -> &str {
        &self.graph_index_inventory_match_report_digest
    }

    pub fn has_admitted_access_plan(&self) -> bool {
        !self.admitted_plan_digest.is_empty()
            && self.admission_posture != WorthQueryGraphReadAccessAdmissionPosture::Denied
    }

    pub(in crate::runtime) fn from_execution_parts(
        read_graph_digest: &str,
        plan: &WorthQueryAdmittedGraphReadAccessPlan,
        plan_consumption: &WorthQueryGraphReadAccessPlanConsumption,
    ) -> Self {
        let admission = plan.admission();
        let digest = hash_parts(&[
            "worth_query_graph_read_access_receipt_summary_v1".to_string(),
            format!("read_graph:{read_graph_digest}"),
            format!("plan:{}", plan.digest()),
            format!("admission:{}", admission.digest()),
            format!("consumption:{}", plan_consumption.digest()),
            format!(
                "authority_receipt:{}",
                admission.authority_receipt().digest()
            ),
            format!("posture:{}", admission.posture().as_str()),
            format!("strategy:{}", plan.execution_strategy()),
            format!(
                "requirements:{}",
                admission.requirement_set().digest().render_support_hex()
            ),
            format!(
                "inventory_match:{}",
                admission.graph_index_inventory_match_report().digest()
            ),
        ]);
        Self {
            digest,
            read_graph_digest: read_graph_digest.to_string(),
            admitted_plan_digest: plan.digest().to_string(),
            admission_digest: admission.digest().to_string(),
            plan_consumption_digest: plan_consumption.digest().to_string(),
            authority_receipt_digest: admission.authority_receipt().digest().to_string(),
            admission_posture: admission.posture().clone(),
            execution_strategy: plan.execution_strategy().to_string(),
            requirement_set_digest: admission.requirement_set().digest().render_support_hex(),
            graph_index_inventory_match_report_digest: admission
                .graph_index_inventory_match_report()
                .digest()
                .to_string(),
        }
    }
}
