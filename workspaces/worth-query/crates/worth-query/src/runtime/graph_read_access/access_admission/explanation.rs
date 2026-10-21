use super::{
    WorthQueryAdmittedGraphReadAccessPlan, WorthQueryGraphReadAccessAdmission,
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadAccessDenial,
    WorthQueryGraphReadAccessInventoryMatch,
};
use crate::runtime::{
    WorthQueryGraphIndexInventoryMatchReport, WorthQueryGraphReadAccessRequirementRow,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessPlanExplanation {
    admission: WorthQueryGraphReadAccessAdmission,
    admitted_plan_digest: Option<String>,
}

impl WorthQueryGraphReadAccessPlanExplanation {
    pub fn admission_digest(&self) -> &str {
        self.admission.digest()
    }

    pub fn admitted_plan_digest(&self) -> Option<&str> {
        self.admitted_plan_digest.as_deref()
    }

    pub fn selected_posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        self.admission.posture()
    }

    pub fn requirement_rows(&self) -> &[WorthQueryGraphReadAccessRequirementRow] {
        self.admission.requirement_set().rows()
    }

    pub fn inventory_matches(&self) -> &[WorthQueryGraphReadAccessInventoryMatch] {
        self.admission.inventory_matches()
    }

    pub fn graph_index_support(&self) -> &WorthQueryGraphIndexInventoryMatchReport {
        self.admission.graph_index_inventory_match_report()
    }

    pub fn denial(&self) -> Option<&WorthQueryGraphReadAccessDenial> {
        self.admission.denial()
    }

    pub(crate) fn from_admission(admission: &WorthQueryGraphReadAccessAdmission) -> Self {
        Self {
            admission: admission.clone(),
            admitted_plan_digest: None,
        }
    }

    pub(crate) fn from_admitted_plan(plan: &WorthQueryAdmittedGraphReadAccessPlan) -> Self {
        Self {
            admission: plan.admission().clone(),
            admitted_plan_digest: Some(plan.digest().to_string()),
        }
    }
}
