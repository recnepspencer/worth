//! One funded owner of pending record release and native cleanup retries.
use super::*;
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure;
/// Completion of this cleanup collection alone, never whole-operation completion.
pub struct WorthQueryOperationCleanupReceipt {
    retired_component_count: usize,
}
impl WorthQueryOperationCleanupReceipt {
    pub fn retired_component_count(&self) -> usize {
        self.retired_component_count
    }
}
/// Owns pending recovery records and failed native releases for funded cleanup retries.
/// Refusal retains remaining custody. Its receipt proves discharge of this collection
/// only, not whole-operation completion or crash durability.
#[derive(Default)]
pub struct WorthQueryOperationCleanup {
    failures: Vec<Failure>,
    pending: Vec<
        worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecovery,
    >,
    retired_component_count: usize,
}
impl WorthQueryOperationCleanup {
    pub(super) fn reserve_release_failure(&mut self) -> Result<(), WorthQueryOperationCloseDenial> {
        self.failures
            .try_reserve(1)
            .map_err(|_| WorthQueryOperationCloseDenial::CleanupCapacity)
    }
    pub(super) fn absorb_release_failure(&mut self, failure: Option<Failure>) {
        if let Some(failure) = failure {
            self.failures.push(failure);
        }
    }
    pub(super) fn from_failures(failures: Vec<Failure>) -> Self {
        Self {
            failures,
            ..Self::default()
        }
    }
    pub fn absorb_recovered(
        &mut self,
        performed: &mut worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryPerformed,
    ) -> Result<(), WorthQueryOperationCloseDenial> {
        self.failures
            .try_reserve(2)
            .map_err(|_| WorthQueryOperationCloseDenial::CleanupCapacity)?;
        let access = worth_query_execution::publication_boundary::program_publication_access();
        if let Some(failure) = performed.take_prior_cleanup_failure(&access) {
            self.failures.push(failure);
        }
        self.pending
            .try_reserve(1)
            .map_err(|_| WorthQueryOperationCloseDenial::CleanupCapacity)?;
        if let Some(pending) = performed.take_pending_cleanup(&access) {
            self.pending.push(pending);
        }
        Ok(())
    }
    pub fn close_in_advancement<S: ApplicationSchema>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        self.failures
            .try_reserve(1)
            .map_err(|_| WorthQueryOperationCloseDenial::CleanupCapacity)?;
        while let Some(record) = self.pending.pop() {
            match runtime.release_product_publication_recovery_in_advancement(phase, record, 0) {
                Ok(receipt) => self.retired_component_count += receipt.retired_component_count(),
                Err(failure) => {
                    let denial = cleanup_denial(&failure);
                    self.failures.push(failure);
                    return Err(denial);
                }
            }
        }
        while let Some(failure) = self.failures.pop() {
            let outcome = match failure {
                Failure::Recovery(failure) => runtime
                    .release_product_publication_recovery_in_advancement(
                        phase,
                        failure.into_recovery(),
                        0,
                    ),
                Failure::OwnerCleanup(failure) => failure
                    .into_cleanup()
                    .retry_in_advancement(phase)
                    .map_err(Failure::OwnerCleanup),
            };
            match outcome {
                Ok(receipt) => self.retired_component_count += receipt.retired_component_count(),
                Err(failure) => {
                    let denial = cleanup_denial(&failure);
                    self.failures.push(failure);
                    return Err(denial);
                }
            }
        }
        Ok(WorthQueryOperationCleanupReceipt {
            retired_component_count: self.retired_component_count,
        })
    }
}
fn cleanup_denial(failure: &Failure) -> WorthQueryOperationCloseDenial {
    match failure {
        Failure::Recovery(failure) => WorthQueryOperationCloseDenial::Recovery(failure.denial()),
        Failure::OwnerCleanup(failure) => WorthQueryOperationCloseDenial::Owner(failure.denial()),
    }
}
