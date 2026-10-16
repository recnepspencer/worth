//! Lawful close retries native cleanup without discarding refused custody.
use super::*;

#[derive(Debug)]
/// Explains why lawful close could not discharge the retained native cleanup obligations.
pub enum WorthQueryOperationCloseDenial {
    FreshRequestMismatch,
    ManagedCleanup(worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailureKind),
    CleanupCapacity,
    NativeObligationRetained,
    Outputs(WorthQueryRequiredOutputPreparationDenial),
    Execution(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Recovery(worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseDenial),
    Owner(worth_query_execution::facade::product::WorthQueryProductBranchOwnerCleanupDenial),
}

/// No source, demand, descriptor or pending cleanup remains owned by this result.
pub struct WorthQueryClosedOperation {
    managed: Option<
        worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt,
    >,
    retired_component_count: usize,
    output_unavailable: Option<WorthQueryApplicationProgramOutputUnavailable>,
}
/// Terminal disposal eligibility only: historical evidence grants no effect retry.
/// Actual managed cleanup and partial publication custody still govern release.
pub(super) fn terminal_outcome_can_close<D, R>(
    outcome: &WorthQueryApplicationMutationOutcome<D, R>,
) -> bool {
    use worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted as Native;
    matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
            | WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_)
            | WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
            | WorthQueryApplicationMutationOutcome::DomainDenied(_)
            | WorthQueryApplicationMutationOutcome::Cancelled
            | WorthQueryApplicationMutationOutcome::DeadlineExceeded
            | WorthQueryApplicationMutationOutcome::Commit(
                Native::Denied(_)
                    | Native::Cancelled
                    | Native::TimedOut
                    | Native::Stale(_)
                    | Native::ProductStale(_)
                    | Native::NoEffect(_)
            )
    )
}

impl WorthQueryClosedOperation {
    pub(super) fn from_cleanup(receipt: WorthQueryOperationCleanupReceipt) -> Self {
        Self {
            retired_component_count: receipt.retired_component_count(),
            managed: receipt.into_managed_receipt(),
            output_unavailable: None,
        }
    }
    pub(super) fn with_output_unavailable(
        mut self,
        evidence: Option<WorthQueryApplicationProgramOutputUnavailable>,
    ) -> Self {
        self.output_unavailable = evidence;
        self
    }
    /// Actual native negative completion. It certifies saved source with no output,
    /// never a positive required-output settlement or current output publication.
    pub fn output_unavailable(&self) -> Option<&WorthQueryApplicationProgramOutputUnavailable> {
        self.output_unavailable.as_ref()
    }
    /// Native release evidence can retain logical recovery requirements; closure is no settlement proof.
    pub fn managed_cleanup_receipt(
        &self,
    ) -> Option<&worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt>
    {
        self.managed.as_ref()
    }
    pub const fn retired_component_count(&self) -> usize {
        self.retired_component_count
    }
}

impl<Schema, Intent, Program, Root, Action>
    WorthQuerySettledFixedOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>:
        ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Err((self, WorthQueryOperationCloseDenial::FreshRequestMismatch));
        }
        let runtime = request.application;
        let outcome = runtime.with_application_advancement(request.scope, |phase| {
            if let Some(performed) = self.recovered.as_mut() {
                self.cleanup.absorb_recovered(performed)?;
            }
            self.cleanup
                .close_in_advancement(&phase, runtime)
                .map(WorthQueryClosedOperation::from_cleanup)
        });
        match outcome {
            Ok(Ok(closed)) => Ok(closed),
            Ok(Err(denial)) => Err((self, denial)),
            Err(denial) => Err((self, WorthQueryOperationCloseDenial::Execution(denial))),
        }
    }
}
