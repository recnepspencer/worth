//! Immutable performed input custody is separate from rebased currentness facts.
use super::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
impl WorthQueryApplicationCommitReceipt {
    pub(in crate::domain_computation::primary_graph) fn performed_output_fields(&self)
    -> Option<&Result<FactPayload, crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure>>{
        self.performed_output_fields.as_ref()
    }
    pub(in crate::domain_computation::primary_graph) fn performed_input_facts(
        &self,
    ) -> Option<&FactPayload> {
        self.performed_input_facts.as_ref()
    }
}
