//! Lawful close retries native cleanup without discarding refused custody.
use super::*;

#[derive(Debug)]
/// Explains why lawful close could not discharge the retained native cleanup obligations.
pub enum WorthQueryOperationCloseDenial {
    FreshRequestMismatch,
    CleanupCapacity,
    NativeObligationRetained,
    Execution(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Recovery(worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseDenial),
    Owner(worth_query_execution::facade::product::WorthQueryProductBranchOwnerCleanupDenial),
}

/// No source, demand, descriptor or pending cleanup remains owned by this result.
pub struct WorthQueryClosedOperation {
    retired_component_count: usize,
}
impl WorthQueryClosedOperation {
    pub(super) fn without_native_obligations() -> Self {
        Self {
            retired_component_count: 0,
        }
    }
    pub(super) fn from_cleanup(retired_component_count: usize) -> Self {
        Self {
            retired_component_count,
        }
    }
    pub const fn retired_component_count(&self) -> usize {
        self.retired_component_count
    }
}

impl<'runtime, Schema, Intent, Program, Root, Action>
    WorthQuerySettledFixedOperation<'runtime, Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
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
        let runtime = self.runtime.runtime();
        let outcome = runtime.with_application_advancement(request.scope, |phase| {
            if let Some(performed) = self.recovered.as_mut() {
                self.cleanup.absorb_recovered(performed)?;
            }
            self.cleanup
                .close_in_advancement(&phase, runtime)
                .map(|receipt| {
                    WorthQueryClosedOperation::from_cleanup(receipt.retired_component_count())
                })
        });
        match outcome {
            Ok(Ok(closed)) => Ok(closed),
            Ok(Err(denial)) => Err((self, denial)),
            Err(denial) => Err((self, WorthQueryOperationCloseDenial::Execution(denial))),
        }
    }
}
