use super::{
    recovery::recovery_failure, WorthQueryProductUnpublishedRecovery,
    WorthQueryProductUnpublishedRecoveryReleaseDenial,
    WorthQueryProductUnpublishedRecoveryReleaseFailure,
};
use crate::domain_computation::execution_runtime::product_world::OwnerCleanupRetirements;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use std::num::NonZeroUsize;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryCursor, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Bounded live discovery from the same World catalog. Vacancies consume
    /// the work bound, so an empty page can still have a continuation.
    pub fn product_publication_recovery_page(
        &self,
        after: Option<&RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
    ) -> Result<RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>>
    {
        self.runtime
            .product_runtime
            .owner
            .inspection_port()
            .recovery_page(after, maximum)
    }

    /// A descriptive handle is admitted only by this root's owning World.
    pub fn readmit_product_publication_recovery(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<WorthQueryProductUnpublishedRecovery, RuntimeWorldRecoveryDenial> {
        let recovery = self.runtime.product_runtime.owner.recovery_port();
        match recovery.inspect_effects(handle) {
            Ok(effects) => drop(effects),
            Err(RuntimeWorldRecoveryDenial::RecoveredCustodyRequired) => {
                drop(recovery.inspect_recovered_effects(handle)?)
            }
            Err(cause) => return Err(cause),
        }
        Ok(WorthQueryProductUnpublishedRecovery::new(
            handle.clone(),
            recovery,
            self.runtime
                .primary_provider
                .unpublished_idempotency_disposition(),
        ))
    }

    /// Releases one World recovery record only after reserving bounded Query
    /// cleanup custody. No raw owner-retirement work crosses this facade.
    pub fn release_product_publication_recovery(
        &self,
        recovery: WorthQueryProductUnpublishedRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryProductUnpublishedRecoveryReleaseFailure,
    >{
        let mut retirements = OwnerCleanupRetirements::default();
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        self.release_product_recovery_in_request(
            &worth_execution::ExecutionRequest::serial(&host),
            recovery,
            minimum_age_ticks,
            &mut retirements,
        )
    }

    pub fn release_product_publication_recovery_in_advancement(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        minimum_age_ticks: u64,
    ) -> Result<crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductUnpublishedRecoveryReleaseFailure>{
        let mut retirements = OwnerCleanupRetirements::default();
        let execution = match phase.checked_execution_request_for(&self.runtime.product_runtime) {
            Ok(execution) => execution,
            Err(denial) => {
                return Err(recovery_failure(
                    recovery,
                    WorthQueryProductUnpublishedRecoveryReleaseDenial::Execution(denial),
                ))
            }
        };
        self.release_product_recovery_in_request(
            &execution,
            recovery,
            minimum_age_ticks,
            &mut retirements,
        )
    }

    pub(crate) fn release_product_recovery_in_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        minimum_age_ticks: u64,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductUnpublishedRecoveryReleaseFailure>{
        use crate::domain_computation::primary_graph::application_contribution::execution_denial;
        let mut retained = Some(recovery);
        let mut attempted = None;
        let outcome = execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    attempted = Some((|| {
                        context.checkpoint(1).map_err(|stop| {
                            recovery_failure(
                                retained.take().expect("one refused release"),
                                WorthQueryProductUnpublishedRecoveryReleaseDenial::Execution(
                                    execution_denial::advancement_checkpoint_stop(stop),
                                ),
                            )
                        })?;
                        self.prepare_product_publication_recovery_release(
                            execution,
                            retained.take().expect("one admitted record release"),
                            minimum_age_ticks,
                        )?
                        .retry_in_request_with_retirements(execution, retirements)
                        .map_err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup)
                    })());
                })
        });
        // Once record release ran, its actual result owns either remaining
        // recovery, pending cleanup or a complete receipt across the final poll.
        if let Some(actual) = attempted {
            return actual;
        }
        match outcome {
            Ok(Err(denial)) | Err(denial) => Err(recovery_failure(
                retained.expect("unstarted release retains its record"),
                WorthQueryProductUnpublishedRecoveryReleaseDenial::Execution(
                    execution_denial::advancement_scope_stop(denial),
                ),
            )),
            Ok(Ok(_)) => unreachable!("completed release callback ran"),
        }
    }

    fn prepare_product_publication_recovery_release(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        minimum_age_ticks: u64,
    ) -> Result<crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup, WorthQueryProductUnpublishedRecoveryReleaseFailure>{
        let handle = recovery.record_handle().clone();
        let provider_release = match self
            .runtime
            .primary_provider
            .unpublished_idempotency_disposition()
            .reserve_release(&handle)
        {
            Ok(reservation) => reservation,
            Err(()) => {
                return Err(recovery_failure(
                    recovery,
                    WorthQueryProductUnpublishedRecoveryReleaseDenial::ProviderBusy,
                ))
            }
        };
        let effects = match self
            .runtime
            .product_runtime
            .owner
            .recovery_port()
            .inspect_effects(&handle)
        {
            Ok(effects) => effects,
            Err(RuntimeWorldRecoveryDenial::RecoveredCustodyRequired) => {
                return self.prepare_recovered_product_release(execution, recovery)
            }
            Err(denial) => return Err(recovery_failure(recovery, denial.into())),
        };
        // The actual World durable-open custody selects reopening before any
        // Memory-only Native observation or binding allocation is attempted.
        if effects
            .next_actions()
            .contains(&worth_runtime_world::facade::ProductUnpublishedNextAction::ReopenRequired)
        {
            return Err(recovery_failure(
                recovery,
                RuntimeWorldRecoveryDenial::ReopenRequired.into(),
            ));
        }
        let (native, original_binding) = match self
            .runtime
            .primary_provider
            .prepare_memory_released_binding(
                execution,
                effects.component_results().relational_settlement(),
                provider_release.is_some(),
            ) {
            Ok(prepared) => prepared,
            Err(cause) => {
                return Err(recovery_failure(
                    recovery,
                    WorthQueryProductUnpublishedRecoveryReleaseDenial::OriginalBinding(cause),
                ))
            }
        };
        if let (Some(reservation), Some(binding)) = (&provider_release, &original_binding) {
            if reservation
                .validate_original(binding, effects.expected_head())
                .is_err()
            {
                return Err(recovery_failure(
                    recovery,
                    WorthQueryProductUnpublishedRecoveryReleaseDenial::OriginalBindingMismatch,
                ));
            }
        }
        let reservation = if effects.destination_branch().is_some() {
            self.runtime
                .product_runtime
                .reserve_owner_cleanup_for_creation()
        } else {
            self.runtime
                .product_runtime
                .reserve_owner_cleanup_for_application()
        };
        drop(effects);
        let reservation = match reservation {
            Ok(reservation) => reservation,
            Err(_) => {
                return Err(recovery_failure(
                    recovery,
                    WorthQueryProductUnpublishedRecoveryReleaseDenial::CleanupCapacityExhausted,
                ))
            }
        };
        let (handle, recovery_port, disposition) = recovery.into_release_parts();
        let cleanup =
            match recovery_port.release_effects(execution, &handle, minimum_age_ticks, native) {
                Ok(cleanup) => cleanup,
                Err(denial) => {
                    return Err(recovery_failure(
                        WorthQueryProductUnpublishedRecovery::new(
                            handle,
                            recovery_port,
                            disposition,
                        ),
                        denial.into(),
                    ))
                }
            };
        let (original, cleanup) = cleanup.into_parts();
        if let Some(provider_release) = provider_release {
            provider_release.finish(original_binding, original);
        }
        let cleanup_identity = reservation.install_unpublished(cleanup);
        Ok(crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup::new(
            self.runtime.product_runtime.clone(),
            cleanup_identity,
        ))
    }
}
