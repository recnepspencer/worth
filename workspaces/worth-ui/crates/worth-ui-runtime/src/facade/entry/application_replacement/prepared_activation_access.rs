use super::{WorthUiApplicationCutoverTransition, WorthUiPreparedApplicationActivation};

impl WorthUiPreparedApplicationActivation {
    pub(in crate::facade::entry) fn candidate_replacement_authority(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority {
        self.prepared_transition()
            .candidate_replacement_authority()
            .expect("application replacement owns its prepared successor")
    }

    pub(super) const fn candidate_service_policy_plan(
        &self,
    ) -> crate::declaration::UiNormalizedServicePolicyPlan {
        self.candidate_service_policy_plan
    }

    pub(super) fn visual_trace_source(
        &self,
    ) -> crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource {
        self.visual_trace_source.clone()
    }

    pub(in crate::facade::entry) fn candidate_plan(
        &self,
    ) -> &crate::runtime::WorthUiActiveExecutionPlan {
        self.prepared_transition().candidate_plan()
    }

    pub(super) fn candidate_query_binding(
        &self,
    ) -> &worth_ui_query_binding::WorthUiRuntimeQueryBinding {
        self.prepared_transition().candidate_query_binding()
    }

    pub(super) fn candidate_allocation_catalog(
        &self,
    ) -> crate::runtime::UiMountedAllocationProjectionCatalog {
        self.prepared_transition().candidate_allocation_catalog()
    }

    pub(super) fn candidate_plan_digest(&self) -> u64 {
        self.prepared_transition().candidate_plan_digest()
    }

    pub(super) fn candidate_allocation_truth_revision(&self) -> u64 {
        self.prepared_transition()
            .candidate_allocation_truth_revision()
    }

    fn prepared_transition(&self) -> &crate::runtime::WorthUiPreparedApplicationPlanSwap {
        match self
            .transition
            .as_ref()
            .expect("prepared application transition is present")
        {
            WorthUiApplicationCutoverTransition::Prepared(activation) => activation,
            WorthUiApplicationCutoverTransition::Committed { .. } => {
                unreachable!("prepared application transition cannot already be committed")
            }
        }
    }
}
