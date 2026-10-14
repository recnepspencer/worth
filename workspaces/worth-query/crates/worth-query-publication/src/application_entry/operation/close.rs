//! Lawful close retries native cleanup without discarding refused custody.
use super::*;
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure;

#[derive(Debug)]
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
            // Reserve before removing a refused native cleanup from its owner.
            self.cleanup
                .try_reserve(1)
                .map_err(|_| WorthQueryOperationCloseDenial::CleanupCapacity)?;
            if let Some(performed) = self.recovered.as_mut() {
                if let Some(failure) = performed.take_prior_cleanup_failure(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                ) {
                    self.cleanup.push(failure);
                }
            }
            if let Some(performed) = self.recovered.as_mut() {
                if let Some(record) = performed.take_pending_cleanup(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                ) {
                    match runtime
                        .release_product_publication_recovery_in_advancement(&phase, record, 0)
                    {
                        Ok(receipt) => {
                            self.retired_component_count += receipt.retired_component_count()
                        }
                        Err(failure) => {
                            let denial = cleanup_denial(&failure);
                            self.cleanup.push(failure);
                            return Err(denial);
                        }
                    }
                }
            }
            while let Some(failure) = self.cleanup.pop() {
                let outcome = match failure {
                    Failure::Recovery(failure) => runtime
                        .release_product_publication_recovery_in_advancement(
                            &phase,
                            failure.into_recovery(),
                            0,
                        ),
                    Failure::OwnerCleanup(failure) => failure
                        .into_cleanup()
                        .retry_in_advancement(&phase)
                        .map_err(Failure::OwnerCleanup),
                };
                match outcome {
                    Ok(receipt) => {
                        self.retired_component_count += receipt.retired_component_count()
                    }
                    Err(failure) => {
                        let denial = cleanup_denial(&failure);
                        self.cleanup.push(failure);
                        return Err(denial);
                    }
                }
            }
            Ok(WorthQueryClosedOperation {
                retired_component_count: self.retired_component_count,
            })
        });
        match outcome {
            Ok(Ok(closed)) => Ok(closed),
            Ok(Err(denial)) => Err((self, denial)),
            Err(denial) => Err((self, WorthQueryOperationCloseDenial::Execution(denial))),
        }
    }
}

fn cleanup_denial(failure: &Failure) -> WorthQueryOperationCloseDenial {
    match failure {
        Failure::Recovery(failure) => WorthQueryOperationCloseDenial::Recovery(failure.denial()),
        Failure::OwnerCleanup(failure) => WorthQueryOperationCloseDenial::Owner(failure.denial()),
    }
}
