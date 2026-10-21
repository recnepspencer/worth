use super::{
    WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadAccessPlanExplanation,
};
use crate::identity::hash_parts;
use crate::runtime::{WorthQueryEphemeralGraphIndexPlan, WorthQueryGraphIndexInventoryMatchReport};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedGraphReadAccessPlan {
    digest: String,
    admission: WorthQueryGraphReadAccessAdmission,
    execution_strategy: String,
    ephemeral_index_plan: Option<WorthQueryEphemeralGraphIndexPlan>,
}

impl WorthQueryAdmittedGraphReadAccessPlan {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn admission(&self) -> &WorthQueryGraphReadAccessAdmission {
        &self.admission
    }

    pub fn posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        self.admission.posture()
    }

    pub fn execution_strategy(&self) -> &str {
        &self.execution_strategy
    }

    pub fn ephemeral_index_plan(&self) -> Option<&WorthQueryEphemeralGraphIndexPlan> {
        self.ephemeral_index_plan.as_ref()
    }

    pub fn graph_index_support(&self) -> &WorthQueryGraphIndexInventoryMatchReport {
        self.admission.graph_index_inventory_match_report()
    }

    pub fn explanation(&self) -> WorthQueryGraphReadAccessPlanExplanation {
        WorthQueryGraphReadAccessPlanExplanation::from_admitted_plan(self)
    }

    pub(crate) fn from_admission(admission: WorthQueryGraphReadAccessAdmission) -> Option<Self> {
        if !admission.is_admitted() {
            return None;
        }
        let execution_strategy = match admission.posture() {
            WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed => {
                "inline-indexed-read-execution"
            }
            WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex => {
                "bounded-ephemeral-index-read-execution"
            }
            WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired
            | WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired
            | WorthQueryGraphReadAccessAdmissionPosture::StoreBackedCapabilityRequired
            | WorthQueryGraphReadAccessAdmissionPosture::AccessCapabilityRegistrationRequired
            | WorthQueryGraphReadAccessAdmissionPosture::Denied => return None,
        }
        .to_string();
        let ephemeral_index_plan = WorthQueryEphemeralGraphIndexPlan::from_admission(&admission);
        let digest = hash_parts(&[
            "worth_query_admitted_graph_read_access_plan_v1".to_string(),
            format!("admission:{}", admission.digest()),
            format!("posture:{}", admission.posture().as_str()),
            format!("strategy:{execution_strategy}"),
            format!(
                "ephemeral_index_plan:{}",
                ephemeral_index_plan
                    .as_ref()
                    .map(WorthQueryEphemeralGraphIndexPlan::digest)
                    .unwrap_or("none")
            ),
        ]);
        Some(Self {
            digest,
            admission,
            execution_strategy,
            ephemeral_index_plan,
        })
    }
}
