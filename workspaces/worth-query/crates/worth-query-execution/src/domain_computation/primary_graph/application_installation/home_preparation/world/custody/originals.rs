//! Rejoin actual World originals with their owning Query operation family.
use super::*;
impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static, Program: 'static>
    ReadmittedHomeQueryService<Schema, Program>
{
    pub(super) fn admit_original_custody(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryHomeCustodyReadmissionDenial> {
        use QueryHomeCustodyReadmissionDenial as Denial;
        let application = self.runtime.runtime();
        let owner = &application.runtime.product_runtime.owner;
        let snapshot = owner
            .inspection_port()
            .recovery_snapshot()
            .map_err(|cause| Denial::World(RuntimeWorldServiceDenial::OwnerUnavailable(cause)))?;
        if snapshot.reserved() != 0 || snapshot.abandoned() != 0 || snapshot.updating() != 0 {
            return Err(Denial::UnclassifiedWorldCustody);
        }
        let recovery = owner.recovery_port();
        let mut after = None;
        loop {
            execution.consult().map_err(Denial::Execution)?;
            let page = owner
                .inspection_port()
                .recovery_page(
                    after.as_ref(),
                    std::num::NonZeroUsize::new(32).expect("nonzero page"),
                )
                .map_err(Denial::World)?;
            for row in page.rows() {
                match row.state() {
                    RuntimeWorldRecoveryRecordState::Recovered => {}
                    RuntimeWorldRecoveryRecordState::Active
                    | RuntimeWorldRecoveryRecordState::Abandoned
                    | RuntimeWorldRecoveryRecordState::Retained
                    | RuntimeWorldRecoveryRecordState::Busy => {
                        return Err(Denial::UnclassifiedWorldCustody)
                    }
                }
                let effects = recovery
                    .inspect_recovered_effects(row.handle())
                    .map_err(Denial::Recovery)?;
                if let Some(support) = application.installed_program_support() {
                    if support
                        .lifecycle()
                        .is_adoption_original(&effects)
                        .map_err(|_| Denial::UnclassifiedWorldCustody)?
                    {
                        if effects.native_receipt().is_none() {
                            let actual = effects.attempt_identity();
                            let attempt=worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(actual.owner_identity().durable().expect("Home original"),actual.ordinal());
                            self.release_empty_original(execution, effects)?;
                            support
                                .lifecycle()
                                .release_branch_adoption(execution, attempt)
                                .map_err(|_| Denial::UnclassifiedWorldCustody)?;
                        }
                        // Native-bearing adoption remains in the real catalogue.
                        continue;
                    }
                }
                if effects.native_receipt().is_none() {
                    self.release_empty_original(execution, effects)?;
                    continue;
                }
                if let Some((entity, completion)) = application
                    .runtime
                    .primary_provider
                    .recovered_inbound_completion_row(execution, &effects)
                    .map_err(|_| Denial::OriginalBindingUnavailable)?
                {
                    application
                        .admit_recovered_inbound_completion(execution, &effects, entity, completion)
                        .map_err(Denial::InboundCustody)?;
                    continue;
                }
                application
                    .runtime
                    .primary_provider
                    .admit_recovered_idempotency(execution, row.handle().clone(), &effects)
                    .map_err(original_binding_denial)?;
            }
            after = page.next_after().cloned();
            if after.is_none() {
                break;
            }
        }
        let mut after = None;
        loop {
            let page = recovery
                .released_original_page(
                    execution,
                    after.as_ref(),
                    std::num::NonZeroUsize::new(32).expect("nonzero page"),
                )
                .map_err(Denial::ReleasedSource)?;
            for original in page.rows() {
                if let Some(support) = application.installed_program_support() {
                    if support
                        .lifecycle()
                        .is_released_adoption_original(original)
                        .map_err(|_| Denial::UnclassifiedWorldCustody)?
                    {
                        continue;
                    }
                }
                if original.native_receipt().is_none() {
                    continue;
                }
                if application
                    .runtime
                    .primary_provider
                    .released_inbound_completion_row(execution, original)
                    .map_err(|_| Denial::OriginalBindingUnavailable)?
                    .is_some()
                {
                    // A released internal completion is not an application key
                    // and is not canonical World completion. Accepted custody
                    // remains retryable against its actual original outbox.
                    continue;
                }
                application
                    .runtime
                    .primary_provider
                    .admit_released_idempotency(execution, original)
                    .map_err(original_binding_denial)?;
            }
            after = page.next_after().cloned();
            if after.is_none() {
                return Ok(());
            }
        }
    }
}

fn original_binding_denial(
    cause: crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial,
) -> QueryHomeCustodyReadmissionDenial {
    use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial as Source;
    use QueryHomeCustodyReadmissionDenial as Denial;
    match cause {
        Source::ForeignAffinity => Denial::ForeignOriginalAffinity,
        Source::RetentionCapacityExhausted | Source::RetentionIdentityExhausted => {
            Denial::OriginalMemoryAdmission
        }
        Source::PreparationBusy
        | Source::ActiveSnapshotCapacityExhausted { .. }
        | Source::SnapshotIdentityExhausted
        | Source::RecordedIntentUnverifiable
        | Source::ExecutionDenied(_)
        | Source::Unavailable
        | Source::NativeKeyLookup { .. } => Denial::OriginalBindingUnavailable,
    }
}
