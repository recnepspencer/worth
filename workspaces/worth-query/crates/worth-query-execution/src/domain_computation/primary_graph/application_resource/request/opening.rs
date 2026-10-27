//! Keep callback output custody even when the original advancement handoff stops.
use super::super::stop::WorthQueryResourceActionDenial;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial;

impl<Schema: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, Schema> {
    pub(super) fn open<T>(
        &self,
        body: impl FnOnce(&worth_execution::ExecutionRequest<'_, '_>) -> Result<T, ResourceCause>,
    ) -> Result<T, ResourceCause> {
        let mut completed = None;
        let handoff = self
            .runtime
            .with_application_advancement(self.request, |phase| {
                completed = Some(
                    match phase.checked_execution_request_for(&self.runtime.product_runtime) {
                        Ok(execution) => body(&execution),
                        Err(cause) => Err(ResourceCause::Advancement(cause)),
                    },
                );
            });
        match (handoff, completed) {
            (Ok(()), Some(result)) => result,
            (Err(WorthQueryAdvancementDenial::NestedStopped), Some(Err(cause))) => Err(cause),
            (Err(cause), _) => Err(ResourceCause::Advancement(cause)),
            (Ok(()), None) => {
                unreachable!("successful original advancement invokes Resource action")
            }
        }
    }

    pub(super) fn denial(cause: ResourceCause) -> WorthQueryResourceActionDenial {
        cause.into()
    }
}
