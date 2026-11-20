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
    pub(in crate::domain_computation::primary_graph) fn retain_authored_prefix_after_own(
        &self,
        root: &WorthQueryProviderSessionTerminalBinding,
        identity: worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity,
        binding: &'static str,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<(), Denial> {
        self.attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .query_support_root(root)?
            .replace_authored_prefix_after_own(identity, binding, pin);
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn retain_candidate_read_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: &WorthQueryProviderSessionTerminalBinding,
        source: EntityId,
        position: usize,
        owner: Option<usize>,
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
            .retain_candidate_read_support(execution, source, position, owner, support)
    }
    pub(in crate::domain_computation::primary_graph) fn supersede_owned_candidate_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: &WorthQueryProviderSessionTerminalBinding,
        owner: usize,
        invalidator: usize,
    ) -> Result<(), Denial> {
        self.attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .query_support_root(root)?
            .supersede_owned_candidate_support(execution, owner, invalidator)
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
    pub(in crate::domain_computation::primary_graph) fn refresh_outgoing_read_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: &WorthQueryProviderSessionTerminalBinding,
    ) -> Result<(), Denial> {
        self.attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .query_support_root(root)?
            .refresh_outgoing_read_support(execution)
    }
    /// Requalify the verified parent the root retained as `read` (PR C R2):
    /// compare it exactly at `current` against the edit's C basis.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn requalify_retained_parent(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: &WorthQueryProviderSessionTerminalBinding,
        before: &crate::basis::WorthQueryProductBranchLease,
        read: crate::domain_computation::primary_graph::provider::ParentRead,
        current: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        invalidator: usize,
        settles: impl Fn(&crate::domain_computation::primary_graph::application_contribution::producer::VerifiedAppliedParent) -> bool,
    ) -> Result<
        crate::domain_computation::primary_graph::provider::RetainedReadRequalification,
        Denial,
    > {
        self.graph.with_runtime(execution, |runtime| {
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                runtime,
                before.relational_basis(),
            )
            .map_err(|_| Denial::new(Kind::StorageUnavailable, "coupled qualification C basis"))?;
            let answer = self
                .attempts
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .query_support_root(root)
                .and_then(|attempt| {
                    attempt.requalify_retained_parent(
                        execution,
                        runtime,
                        &snapshot,
                        read,
                        current,
                        invalidator,
                        settles,
                    )
                });
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            answer
        })
    }
}
