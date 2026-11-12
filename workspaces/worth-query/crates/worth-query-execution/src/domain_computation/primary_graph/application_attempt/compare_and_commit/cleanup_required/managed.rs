//! Actual native cleanup state stays in its originally prepaid slot.
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
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
        let mut owner = Some(self);
        let result = execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    context
                        .checkpoint(0)
                        .map_err(execution_denial::advancement_checkpoint_stop)?;
                    let Self {
                        mut state,
                        return_owner,
                    } = owner
                        .take()
                        .expect("checked cleanup owns its native terminal");
                    let ManagedCleanupState {
                        failure,
                        original,
                        runtime,
                        bridge,
                        ..
                    } = state.take().expect("native cleanup owns initialized state");
                    Ok(match failure.retry() {
                        Ok(native) => {
                            let receipt =
                                super::super::WorthQueryApplicationManagedCleanupReceipt::new(
                                    native, original,
                                );
                            drop(state);
                            drop(return_owner);
                            Ok(receipt)
                        }
                        Err(failure) => {
                            *state = Some(ManagedCleanupState {
                                failure,
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
                    })
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
}
impl std::fmt::Debug for WorthQueryApplicationManagedCleanup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state().failure.fmt(f)
    }
}
