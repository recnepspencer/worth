//! Business evidence and the real failed cleanup owner travel together.
use super::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::{
    execution_runtime::product_world::WorthQueryProductRuntime, WorthQueryMutationCleanupFailure,
};

/// Original business evidence and actual failed native cleanup travel together.
#[derive(Debug)]
#[must_use = "retain business evidence and native cleanup custody"]
pub struct WorthQueryApplicationCleanupRequired {
    outcome: WorthQueryApplicationCommitOutcome,
    cleanup: WorthQueryApplicationManagedCleanup,
}
impl WorthQueryApplicationCleanupRequired {
    pub(in crate::domain_computation::primary_graph) fn new(
        outcome: WorthQueryApplicationCommitOutcome,
        failure: WorthQueryMutationCleanupFailure,
        runtime: WorthQueryProductRuntime,
        original: Option<super::RetainedOriginalCleanupCommit>,
        bridge: std::sync::Arc<
            std::sync::RwLock<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>,
        >,
    ) -> Self {
        Self {
            outcome,
            cleanup: WorthQueryApplicationManagedCleanup {
                failure,
                original,
                runtime,
                last_denial: None,
                bridge,
            },
        }
    }
    pub(in crate::domain_computation::primary_graph) fn from_parts(
        outcome: WorthQueryApplicationCommitOutcome,
        cleanup: WorthQueryApplicationManagedCleanup,
    ) -> Box<Self> {
        Box::new(Self { outcome, cleanup })
    }
    /// Business evidence retains its native outcome independently of failed cleanup.
    pub fn outcome(&self) -> &WorthQueryApplicationCommitOutcome {
        &self.outcome
    }
    /// Borrows the actual native cleanup owner without transferring authority.
    pub fn cleanup(&self) -> &WorthQueryApplicationManagedCleanup {
        &self.cleanup
    }
    /// Transfers both owners without manufacturing business completion.
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationManagedCleanup,
    ) {
        (self.outcome, self.cleanup)
    }
}

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
    failure: WorthQueryMutationCleanupFailure,
    original: Option<super::RetainedOriginalCleanupCommit>,
    runtime: WorthQueryProductRuntime,
    bridge: std::sync::Arc<std::sync::RwLock<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>>,
    last_denial: Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial>,
}
impl WorthQueryApplicationManagedCleanup {
    /// Borrows the exact failed managed-run owner.
    pub fn failure(&self) -> &WorthQueryMutationCleanupFailure {
        &self.failure
    }
    /// The most recent fresh issuer or execution refusal, if retry stopped before native cleanup.
    pub fn last_retry_denial(&self) -> Option<&crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial>{
        self.last_denial.as_ref()
    }
    /// Retries native cleanup under this exact runtime's fresh bounded issuer; refusal returns this owner.
    pub fn retry_in_advancement(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<super::WorthQueryApplicationManagedCleanupReceipt, Self> {
        use crate::domain_computation::primary_graph::application_contribution::execution_denial;
        let execution = match phase.checked_execution_request_for(&self.runtime) {
            Ok(execution) => execution,
            Err(denial) => {
                self.last_denial = Some(denial);
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
                        failure,
                        original,
                        runtime,
                        bridge,
                        ..
                    } = owner
                        .take()
                        .expect("checked cleanup owns its native terminal");
                    Ok(match failure.retry() {
                        Ok(native) => Ok(super::WorthQueryApplicationManagedCleanupReceipt::new(
                            native, original,
                        )),
                        Err(failure) => Err(Self {
                            failure,
                            original,
                            runtime,
                            bridge,
                            last_denial: None,
                        }),
                    })
                })
        });
        match result {
            Ok(Ok((Ok(result), _))) => result,
            Ok(Ok((Err(denial), _))) => {
                let mut owner = owner.expect("checkpoint refusal precedes native cleanup");
                owner.last_denial = Some(denial);
                Err(owner)
            }
            Ok(Err(stop)) | Err(stop) => {
                let mut owner = owner.expect("scope refusal precedes native cleanup");
                owner.last_denial = Some(execution_denial::advancement_scope_stop(stop));
                Err(owner)
            }
        }
    }
}
impl std::fmt::Debug for WorthQueryApplicationManagedCleanup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.failure.fmt(f)
    }
}
