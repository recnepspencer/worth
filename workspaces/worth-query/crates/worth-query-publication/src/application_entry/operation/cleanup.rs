//! One funded owner of pending record release and native cleanup retries.
use super::*;
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure;
/// Completion of this cleanup collection alone, never whole-operation completion.
pub struct WorthQueryOperationCleanupReceipt {
    managed: Option<
        worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt,
    >,
    retired_component_count: usize,
}
impl WorthQueryOperationCleanupReceipt {
    /// Actual native cleanup evidence, including any remaining logical recovery disposition.
    pub fn managed_cleanup_receipt(
        &self,
    ) -> Option<&worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt>
    {
        self.managed.as_ref()
    }
    pub(super) fn into_managed_receipt(
        self,
    ) -> Option<worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt>
    {
        self.managed
    }

    pub fn retired_component_count(&self) -> usize {
        self.retired_component_count
    }
}
/// Owns pending recovery records and failed native releases for funded cleanup retries.
/// Refusal retains remaining custody. Its receipt proves discharge of this collection
/// only, not whole-operation completion or crash durability.
#[derive(Default)]
pub struct WorthQueryOperationCleanup {
    // The existing installed allocation remains owned after descriptor abandonment.
    keepalive: Option<Box<dyn std::any::Any + Send + Sync>>,
    managed_receipt: Option<
        worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt,
    >,
    managed:
        Option<worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanup>,
    // A failed output producer may have unpublished or uncertain business custody.
    producer_outcome:
        Option<Box<worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted>>,
    failures: Vec<Failure>,
    pending: Vec<
        worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecovery,
    >,
    retired_component_count: usize,
}
impl WorthQueryOperationCleanup {
    pub(super) fn absorb_producer<S, P>(
        &mut self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        required: Box<
            worth_query_execution::facade::primary_graph::WorthQueryApplicationCleanupRequired,
        >,
    ) where
        S: ApplicationSchema + 'static,
        P: ApplicationProgramDefinition<S>,
        WorthQueryProgramApplicationRuntime<S, P>: Send + Sync + 'static,
    {
        let (outcome, cleanup) = required.into_parts();
        self.producer_outcome = Some(Box::new(outcome));
        self.absorb_managed(runtime, cleanup);
    }
    pub(super) fn absorb_managed<S, P>(
        &mut self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        cleanup: worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanup,
    ) where
        S: ApplicationSchema + 'static,
        P: ApplicationProgramDefinition<S>,
        WorthQueryProgramApplicationRuntime<S, P>: Send + Sync + 'static,
    {
        assert!(
            self.managed.is_none(),
            "one fixed source owns one pending managed cleanup"
        );
        self.keepalive = Some(Box::new(runtime.clone()));
        self.managed = Some(cleanup);
    }
    pub(super) fn absorb_native_outcome<S, P, D, R>(
        &mut self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        outcome: WorthQueryApplicationMutationOutcome<D, R>,
    ) -> WorthQueryApplicationMutationOutcome<D, R>
    where
        S: ApplicationSchema + 'static,
        P: ApplicationProgramDefinition<S>,
        WorthQueryProgramApplicationRuntime<S, P>: Send + Sync + 'static,
    {
        match outcome {
            WorthQueryApplicationMutationOutcome::Commit(worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted::CleanupRequired(required)) if self.managed.is_none() => {
                let (outcome, cleanup) = required.into_parts();
                self.absorb_managed(runtime, cleanup);
                WorthQueryApplicationMutationOutcome::Commit(outcome)
            }
            other => other,
        }
    }
    /// Original output-producer business evidence remains independent of cleanup release.
    pub fn producer_native_outcome(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted>
    {
        self.producer_outcome.as_deref()
    }
    /// Exact retained managed-run failure; this observation grants no cleanup or business authority.
    pub fn managed_failure(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure>
    {
        self.managed.as_ref().map(|cleanup| cleanup.failure())
    }
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
        if let Some(cleanup) = self.managed.take() {
            match cleanup.retry_in_advancement(phase) {
                Ok(receipt) => self.managed_receipt = Some(receipt),
                Err(cleanup) => {
                    let denial = cleanup
                        .last_retry_denial()
                        .copied()
                        .map(WorthQueryOperationCloseDenial::Execution)
                        .unwrap_or_else(|| {
                            WorthQueryOperationCloseDenial::ManagedCleanup(cleanup.failure().kind())
                        });
                    self.managed = Some(cleanup);
                    return Err(denial);
                }
            }
        }
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
        self.keepalive = None;
        Ok(WorthQueryOperationCleanupReceipt {
            managed: self.managed_receipt.take(),
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
