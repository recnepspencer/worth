//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPreparedApplicationProviderAttempt {
    #[doc(hidden)]
    pub fn effects_for_test(&self) -> &effect_accumulator::WorthQueryRegisteredProviderEffects {
        &self.effects
    }
}

impl WorthQueryPreparedApplicationProviderAttempt {
    #[doc(hidden)]
    pub fn into_parts_for_test(self) -> (Option<crate::domain_computation::primary_graph::application_attempt::workflow::operation_output::WorkflowOperationOrigin>, Option<super::super::workflow_transition_program::LocalActionSettlementProof>, Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>, crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::CompletedReadScopes, FactPayload, Option<std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::CompletedFactProjection>>, Option<std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::CompletedOriginalRead>>, Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>, effect_accumulator::WorthQueryRegisteredProviderEffects, Option<InstalledPreImageDemand>, Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>, bool, bool, &'static [crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerInvariantRequirement], Option<super::super::OutputCurrentnessFacts>,){
        (
            self.workflow_origin,
            self.local_action_settlement,
            self.required_output_demand,
            self.installed_read_scopes,
            self.facts,
            self.application_projection,
            self.original_read,
            self.consumed_outputs,
            self.effects,
            self.preimage_demand,
            self.conditional_definition,
            self.retain_output_demand_observation,
            self.retain_client_observation,
            self.producer_required_invariants,
            self.output_currentness_facts,
        )
    }
}

impl WorthQueryPreparedApplicationProviderAttempt {
    #[doc(hidden)]
    pub fn effects_into_for_test(self) -> effect_accumulator::WorthQueryRegisteredProviderEffects {
        self.effects
    }
}
