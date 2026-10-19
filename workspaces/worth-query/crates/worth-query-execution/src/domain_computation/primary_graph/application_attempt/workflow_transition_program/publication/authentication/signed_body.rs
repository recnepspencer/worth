//! The joint candidate owns one event proof; approval never consumes it a second time.
use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum ApprovalAuthenticationMode {
    Standalone,
    SignedBody,
}

impl<Schema> PreparedWorkflowApprovalAuthentication<Schema> {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::publication) fn bind_signed_body(
        &mut self,
        intent: WorthQueryAuthenticationEventIntent,
    ) -> Result<(), WorthQueryAuthenticationEventDenial> {
        if self.consumed.is_some() || self.mode != ApprovalAuthenticationMode::Standalone {
            return Err(WorthQueryAuthenticationEventDenial::EventUnavailable);
        }
        self.mode = ApprovalAuthenticationMode::SignedBody;
        self.intent = intent;
        Ok(())
    }
}
