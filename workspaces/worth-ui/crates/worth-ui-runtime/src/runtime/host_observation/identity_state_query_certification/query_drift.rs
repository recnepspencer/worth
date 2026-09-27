use crate::runtime::{
    WorthUiQueryBindingDriftDenial, WorthUiQueryBindingDriftDenialKind,
    WorthUiQueryLiveRebindOutcome, WorthUiQueryLiveRebindPlan,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiQueryDriftCertification {
    plans: Vec<WorthUiQueryLiveRebindPlan>,
    typed_denials: Vec<WorthUiQueryBindingDriftDenial>,
}

impl WorthUiQueryDriftCertification {
    pub(crate) fn new(plans: Vec<WorthUiQueryLiveRebindPlan>) -> Self {
        let typed_denials = plans
            .iter()
            .flat_map(|plan| plan.entries())
            .filter_map(|entry| match entry.outcome() {
                WorthUiQueryLiveRebindOutcome::Deny(denial) => Some(denial.clone()),
                WorthUiQueryLiveRebindOutcome::Preserve(_)
                | WorthUiQueryLiveRebindOutcome::Rebind(_)
                | WorthUiQueryLiveRebindOutcome::Retire(_) => None,
            })
            .collect::<Vec<_>>();
        Self {
            plans,
            typed_denials,
        }
    }

    pub fn plans(&self) -> &[WorthUiQueryLiveRebindPlan] {
        &self.plans
    }

    pub fn typed_denials(&self) -> &[WorthUiQueryBindingDriftDenial] {
        &self.typed_denials
    }

    pub fn typed_denial_kinds(&self) -> Vec<WorthUiQueryBindingDriftDenialKind> {
        self.typed_denials
            .iter()
            .map(WorthUiQueryBindingDriftDenial::reason)
            .collect()
    }
}
