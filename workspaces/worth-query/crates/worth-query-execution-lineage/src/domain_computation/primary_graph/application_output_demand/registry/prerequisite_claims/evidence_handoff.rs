use super::PreparedPrerequisiteClaims;
use crate::domain_computation::primary_graph::{
    application_attempt::CompletedHandlerFactBoundary,
    output_lineage::{CompletedDecisionReuseProof, PreparedInputReuseKey},
};

impl PreparedPrerequisiteClaims {
    /// Shares the original issued coordinate; it grants no Ready or currentness.
    pub fn original_ready_key(&self) -> std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey>{
        std::sync::Arc::clone(self.context.key_arc())
    }

    pub fn selected_read_predecessor(
        &self,
    ) -> Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication> {
        self.context.selected_read_predecessor()
    }
    pub fn native_publication_family(
        &self,
    ) -> &'static str {
        self.context.publication_family()
    }

    /// Borrow the exact managed producer key; publication pays for its own copy.
    pub fn native_prior_checkpoint_input(
        &self,
    ) -> (&str, [u8; 32]) {
        let key = self.context.key();
        (&key.producer, key.source.checkpoint_identity().bytes())
    }

    pub fn take_completed_handler_facts(
        &mut self,
    ) -> Option<CompletedHandlerFactBoundary> {
        self.context.take_completed_handler_facts()
    }

    pub fn take_sealed_computation(
        &mut self,
    ) -> (
        crate::domain_computation::primary_graph::application_contribution::SealedComputationRetention,
        Option<crate::domain_computation::primary_graph::output_lineage::PriorComputationRecord>,
    ){
        self.context.take_sealed_computation()
    }

    pub fn take_completed_decision_reuse(
        &mut self,
    ) -> Option<CompletedDecisionReuseProof> {
        self.context.take_completed_decision_reuse()
    }

    pub fn take_prepared_input_reuse_key(
        &mut self,
    ) -> Option<PreparedInputReuseKey> {
        self.context.take_prepared_input_reuse_key()
    }
}
