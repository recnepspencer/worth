//! Actual native cleanup state stays in its originally prepaid slot.
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::{
    query_obligations::history::{
        managed_cleanup::ManagedCleanupPublication, QueryObligationHistory,
    },
    query_obligations::QueryObligationDenial,
};
use crate::domain_computation::{
    execution_runtime::product_world::WorthQueryProductRuntime, WorthQueryMutationCleanupFailure,
};
/// Retains the original managed terminal and exact native World/Bridge owners for cleanup only.
/// Standalone native use still requires a lawful live issuer; this is no shutdown or durability proof.
/// A native cleanup receipt cannot become mutation graph-work completion.
/// ```compile_fail,E0308
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanup;
/// use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
/// use worth_query_execution::facade::domain_computation::WorthQueryMutationGraphWorkCompletion;
/// fn not_business_completion(owner: WorthQueryApplicationManagedCleanup, phase: &WorthQueryAdvancementPhase<'_>) -> WorthQueryMutationGraphWorkCompletion {
///     owner.retry_in_advancement(phase).unwrap()
/// }
/// ```
/// The same cleanup owner has a valid native cleanup result.
/// ```
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanup;
/// use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanupReceipt;
/// fn cleanup(owner: WorthQueryApplicationManagedCleanup, phase: &WorthQueryAdvancementPhase<'_>) -> Result<WorthQueryApplicationManagedCleanupReceipt, WorthQueryApplicationManagedCleanup> {
///     owner.retry_in_advancement(phase)
/// }
/// ```
#[must_use = "cleanup refusal returns this original owner"]
pub struct WorthQueryApplicationManagedCleanup {
    pub(super) state: Box<Option<ManagedCleanupState>>,
    pub(super) return_owner: Option<Box<super::WorthQueryApplicationCleanupRequired>>,
}
pub(super) struct ManagedCleanupState {
    pub(super) business_original: Option<crate::domain_computation::primary_graph::query_obligations::QueryManagedCleanupOriginal>,
    pub(super) history: Option<std::sync::Arc<QueryObligationHistory>>,
    pub(super) publication: Option<ManagedCleanupPublication>,
    pub(super) journal_denial: Option<QueryObligationDenial>,
    pub(super) failure: WorthQueryMutationCleanupFailure,
    pub(super) original: Option<super::super::RetainedOriginalCleanupCommit>,
    pub(super) runtime: WorthQueryProductRuntime,
    pub(super) bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
    pub(super) last_denial: Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial>,
}
impl std::fmt::Debug for ManagedCleanupState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.failure.fmt(f)
    }
}

impl WorthQueryApplicationManagedCleanup {
    fn state(&self) -> &ManagedCleanupState {
        self.state
            .as_ref()
            .as_ref()
            .expect("live managed cleanup has initialized state")
    }
    fn state_mut(&mut self) -> &mut ManagedCleanupState {
        self.state
            .as_mut()
            .as_mut()
            .expect("live managed cleanup has initialized state")
    }
    #[cfg(test)]
    pub(in crate::domain_computation) fn observe_inline_budget(
        census: &mut crate::inline_type_sizes::Census,
    ) {
        let _owner=census.owner::<Self>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/cleanup_required/managed.rs","WorthQueryApplicationManagedCleanup",64);
        let _state=census.owner::<ManagedCleanupState>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/cleanup_required/managed.rs","ManagedCleanupState",16*1024);
    }
    pub(super) fn retain_for_home(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        history: std::sync::Arc<QueryObligationHistory>,
        original: Option<crate::domain_computation::primary_graph::query_obligations::QueryManagedCleanupOriginal>,
    ) {
        self.state_mut().business_original = original;
        self.state_mut().history = Some(history);
        if let Err(cause) = self.ensure_durable(request) {
            self.state_mut().journal_denial = Some(cause);
        }
    }
    fn ensure_durable(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        let state = self.state_mut();
        if let Some(publication) = state.publication.as_mut() {
            let result = publication.finish_retention(request);
            if result == Err(QueryObligationDenial::PublicationNoEffect) {
                state.publication = None;
            }
            if result.is_ok() {
                state.journal_denial = None;
            }
            return result;
        }
        let Some(history) = state.history.as_ref() else {
            return Ok(());
        };
        let original = Self::original_association(state);
        state.publication = history.retain_managed_cleanup(request, &state.failure, original)?;
        if let Some(cause) = state
            .publication
            .as_ref()
            .and_then(|publication| publication.pending_denial.clone())
        {
            return Err(cause);
        }
        state.journal_denial = None;
        Ok(())
    }
    fn original_association(
        state: &ManagedCleanupState,
    ) -> Option<
        crate::domain_computation::primary_graph::query_obligations::QueryManagedCleanupOriginal,
    > {
        state.business_original.or_else(|| {
            state.original.as_ref().and_then(|original| {
                crate::domain_computation::primary_graph::application_attempt::business_original::from_attempt(
                    original
                        .original()
                        .original_publication()
                        .publication_attempt(),
                )
            })
        })
    }
    pub fn obligation_identity(
        &self,
    ) -> Option<worth_foundational::facade::ManagedCleanupObligationIdentity> {
        self.state()
            .publication
            .as_ref()
            .filter(|publication| publication.pending_denial.is_none())
            .map(ManagedCleanupPublication::identity)
    }
    pub fn durability_denial(&self) -> Option<&QueryObligationDenial> {
        self.state().journal_denial.as_ref()
    }
    /// Borrows the exact failed managed-run owner.
    pub fn failure(&self) -> &WorthQueryMutationCleanupFailure {
        &self.state().failure
    }
    /// The most recent fresh issuer or execution refusal, if retry stopped before native cleanup.
    pub fn last_retry_denial(&self) -> Option<&crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial>{
        self.state().last_denial.as_ref()
    }
    /// Retries native cleanup under this exact runtime's fresh bounded issuer; refusal returns this owner.
    pub fn retry_in_advancement(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<super::super::WorthQueryApplicationManagedCleanupReceipt, Self> {
        use crate::domain_computation::primary_graph::application_contribution::execution_denial;
        let execution = match phase.checked_execution_request_for(&self.state().runtime) {
            Ok(execution) => execution,
            Err(denial) => {
                self.state_mut().last_denial = Some(denial);
                return Err(self);
            }
        };
        if let Err(cause) = self.ensure_durable(&execution) {
            self.state_mut().journal_denial = Some(cause);
            return Err(self);
        }
        let mut owner = Some(self);
        let result = execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    context
                        .checkpoint(0)
                        .map_err(execution_denial::advancement_checkpoint_stop)?;
                    Ok(owner
                        .take()
                        .expect("checked native cleanup owner")
                        .release_and_record(&execution))
                })
        });
        match result {
            Ok(Ok((Ok(result), _))) => result,
            Ok(Ok((Err(denial), _))) => {
                let mut owner = owner.expect("checkpoint refusal precedes native cleanup");
                owner.state_mut().last_denial = Some(denial);
                Err(owner)
            }
            Ok(Err(stop)) | Err(stop) => {
                let mut owner = owner.expect("scope refusal precedes native cleanup");
                owner.state_mut().last_denial =
                    Some(execution_denial::advancement_scope_stop(stop));
                Err(owner)
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn retry_for_home(
        mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::super::WorthQueryApplicationManagedCleanupReceipt, Self> {
        let admission = request
            .consult()
            .map_err(QueryObligationDenial::Execution)
            .and_then(|_| self.ensure_durable(request));
        if let Err(cause) = admission {
            self.state_mut().journal_denial = Some(cause);
            return Err(self);
        }
        self.release_and_record(request)
    }
    fn release_and_record(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::super::WorthQueryApplicationManagedCleanupReceipt, Self> {
        let Self {
            mut state,
            return_owner,
        } = self;
        let ManagedCleanupState {
            failure,
            business_original,
            history,
            publication,
            journal_denial: _,
            original,
            runtime,
            bridge,
            ..
        } = state.take().expect("native cleanup owns initialized state");
        match failure.retry_retaining_release() {
            Ok((native, retained)) => {
                if let Some(owner_publication) = publication.as_ref() {
                    if let Err(cause) = owner_publication.record_release(&execution, &native) {
                        *state = Some(ManagedCleanupState {
                            failure: retained,
                            business_original,
                            history,
                            publication,
                            journal_denial: Some(cause),
                            original,
                            runtime,
                            bridge,
                            last_denial: None,
                        });
                        return Err(Self {
                            state,
                            return_owner,
                        });
                    }
                }
                let receipt = super::super::WorthQueryApplicationManagedCleanupReceipt::new(
                    native,
                    original,
                    publication
                        .as_ref()
                        .map(ManagedCleanupPublication::identity),
                );
                drop(state);
                drop(return_owner);
                Ok(receipt)
            }
            Err(failure) => {
                let journal_denial = publication
                    .as_ref()
                    .and_then(|publication| publication.record_failure(&execution, &failure).err());
                *state = Some(ManagedCleanupState {
                    failure,
                    business_original,
                    history,
                    publication,
                    journal_denial,
                    original,
                    runtime,
                    bridge,
                    last_denial: None,
                });
                Err(Self {
                    state,
                    return_owner,
                })
            }
        }
    }
}
impl std::fmt::Debug for WorthQueryApplicationManagedCleanup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state().failure.fmt(f)
    }
}
