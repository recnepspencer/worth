pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::managed_application_recovery::producer::*;
use super::*;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn recover_original_producer_publication<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<S, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        retain: &mut dyn FnMut(ProducerRecoveryCustody),
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
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
            RecoveredPublicationDelivery::Producer(retain),
        )
        .map(|(outcome, _)| outcome)
    }
}

/// A fresh recovery caller funds this fixed shared handoff after authorization,
/// before either owner settlement or World movement. Every payload backing is
/// already shared; the authenticated descriptor stays with the later keyed read.
pub fn reserve_original_output(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    delivery: &RecoveredPublicationDelivery<'_>,
) -> Result<
    Option<Option<worth_execution::ExecutionMemoryReservation>>,
    WorthQueryManagedApplicationRecoveryDenial,
> {
    match delivery {
        RecoveredPublicationDelivery::Ordinary
        | RecoveredPublicationDelivery::OwningSource
        | RecoveredPublicationDelivery::ProgramSource => Ok(None),
        RecoveredPublicationDelivery::Producer(_) => {
            use crate::domain_computation::primary_graph::application_output_demand::RecoveredProducerOutputFacts;
            let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
                .extend(std::alloc::Layout::new::<RecoveredProducerOutputFacts>())
                .expect("fixed recovered producer evidence layout")
                .0
                .pad_to_align();
            request
                .reserve_retained_memory(layout.size() as u64)
                .map(Some)
                .map_err(|cause| {
                    WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied(advancement_denial(
                        cause,
                    ))
                })
        }
    }
}
