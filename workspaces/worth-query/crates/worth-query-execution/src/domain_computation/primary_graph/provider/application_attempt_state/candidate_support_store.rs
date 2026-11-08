//! Retain actual discovery evidence on its already registered root, never admit from a descriptor.
use super::*;
use crate::domain_computation::primary_graph::{
    provider::CandidateReadSupport, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind,
};
use crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding;
use worth_relational::facade::identity::EntityId;
impl WorthQueryPrimaryGraphApplicationAttemptStore {
    fn query_support_root(
        &mut self,
        root: &WorthQueryProviderSessionTerminalBinding,
    ) -> Result<&mut WorthQueryPrimaryGraphApplicationAttempt, Denial> {
        let key = WorthQueryApplicationAttemptLookupKey::from_identity(root.affinity_identity());
        let Some(WorthQueryApplicationAttemptEntry::Registered { state, .. }) =
            self.attempts.get_mut(&key)
        else {
            return Err(Denial::new(
                Kind::ForeignPhase,
                "required graph registered root",
            ));
        };
        if !state.attempt().affinity().same_session(root) {
            return Err(Denial::new(
                Kind::ForeignPhase,
                "required graph root affinity",
            ));
        }
        Ok(state.attempt_mut())
    }
}
impl super::super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn retain_candidate_read_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: &WorthQueryProviderSessionTerminalBinding,
        source: EntityId,
        support: CandidateReadSupport,
    ) -> Result<(), Denial> {
        if self
            .publication_fault_control
            .take_query_support_memory_probe()
        {
            let reserved = execution
                .allocation_policy()
                .reserve_memory(u64::MAX)
                .map_err(|cause| Denial::allocation_denied(support.subject(), cause))?;
            drop(reserved);
        }
        self.attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .query_support_root(root)?
            .retain_candidate_read_support(execution, source, support)
    }
    pub(in crate::domain_computation::primary_graph) fn reset_candidate_read_support(
        &self,
        root: &WorthQueryProviderSessionTerminalBinding,
    ) -> Result<(), Denial> {
        self.attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .query_support_root(root)?
            .reset_candidate_read_support();
        Ok(())
    }
}
