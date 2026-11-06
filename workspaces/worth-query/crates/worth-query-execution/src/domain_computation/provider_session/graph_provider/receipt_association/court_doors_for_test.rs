//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryBoundGraphExecutionAssociation {
    #[doc(hidden)]
    pub fn attempt_for_test(&self) -> &WorthQueryClosedExecutionAttemptIdentity {
        &self.attempt
    }
}

impl WorthQueryBoundGraphExecutionAssociation {
    #[doc(hidden)]
    pub fn provider_session_for_test(&self) -> &WorthQueryExecutionProviderSessionIdentity {
        &self.provider_session
    }
}

impl WorthQueryBoundGraphExecutionAssociation {
    #[doc(hidden)]
    pub fn attempt_mut_for_test(&mut self) -> &mut WorthQueryClosedExecutionAttemptIdentity {
        &mut self.attempt
    }
}

impl WorthQueryBoundGraphExecutionAssociation {
    #[doc(hidden)]
    pub fn provider_session_mut_for_test(
        &mut self,
    ) -> &mut WorthQueryExecutionProviderSessionIdentity {
        &mut self.provider_session
    }
}

impl WorthQueryBoundGraphExecutionAssociation {
    #[doc(hidden)]
    pub fn admits_execution_for_test(
        &self,
        authority: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
        session: &crate::domain_computation::WorthQueryExecutionProviderSession,
        stage_identity: Option<&str>,
        execution_snapshot: &crate::domain_computation::domain_evidence_binding::WorthQueryBoundExecutionSnapshotIdentity,
    ) -> bool {
        Self::admits_execution(self, authority, session, stage_identity, execution_snapshot)
    }
}

impl WorthQueryBoundGraphExecutionAssociation {
    #[doc(hidden)]
    pub fn capture_for_test(call: &WorthQueryGraphProviderCall) -> Self {
        Self::capture(call)
    }
}
