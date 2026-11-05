//! Original producer recovery uses the same retained native attempt owner.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource;

pub(in crate::domain_computation::primary_graph) trait ProducerScopedOps<S: ApplicationSchema>:
    crate::domain_computation::primary_graph::provider::host::ProviderHost<S>
{
    fn recover_original_producer_publication<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<S, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        (
            WorthQueryManagedApplicationRecoveryOutcome,
            Option<WorthQueryRecoveredProgramOutputSource>,
        ),
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        // Provenance is not permission. The same engine still validates fresh
        // authority and matches the actual retained attempt's full binding.
        if idempotency.producing_implementation().is_none() {
            return Err(WorthQueryManagedApplicationRecoveryDenial::ForeignAdmission);
        }
        self.recover_admitted_unpublished_application_delivery(
            phase,
            recovery,
            admission,
            idempotency,
            RecoveredPublicationDelivery::ProgramSource,
        )
    }
}

impl<
        S: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::provider::host::ProviderHost<S> + ?Sized,
    > ProducerScopedOps<S> for HostRuntime
{
}
