use super::PreparedPrerequisiteClaims;
use crate::domain_computation::primary_graph::{
    application_attempt::CompletedHandlerFactBoundary,
    output_lineage::{CompletedDecisionReuseProof, PreparedInputReuseKey},
};

impl PreparedPrerequisiteClaims {
    pub(in crate::domain_computation::primary_graph) fn take_actual_resources(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerDemandResources>{
        self.context.take_actual_resources()
    }

    pub(in crate::domain_computation::primary_graph) fn take_completed_handler_facts(
        &mut self,
    ) -> Option<CompletedHandlerFactBoundary> {
        self.context.take_completed_handler_facts()
    }

    pub(in crate::domain_computation::primary_graph) fn take_sealed_computation(
        &mut self,
    ) -> Option<(
        crate::domain_computation::primary_graph::SealedComputationRun,
        Option<crate::domain_computation::primary_graph::output_lineage::PriorComputationRecord>,
    )> {
        self.context.take_sealed_computation()
    }

    pub(in crate::domain_computation::primary_graph) fn take_completed_decision_reuse(
        &mut self,
    ) -> Option<CompletedDecisionReuseProof> {
        self.context.take_completed_decision_reuse()
    }

    pub(in crate::domain_computation::primary_graph) fn take_prepared_input_reuse_key(
        &mut self,
    ) -> Option<PreparedInputReuseKey> {
        self.context.take_prepared_input_reuse_key()
    }
}
