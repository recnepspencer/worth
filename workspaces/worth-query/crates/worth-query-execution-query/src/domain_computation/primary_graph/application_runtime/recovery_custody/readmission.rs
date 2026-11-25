//! Fresh admission joins actual World/Native originals and retained original input.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution, WorthQueryCommittedProductPublication,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_attempt::facade::ports::RecoveryHandleBindingInternalPort as _;
use worth_query_execution_lineage::facade::ports::CommittedProductPublicationInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Reissue capability for the original claim, never from a wire recovery token.
    pub fn mint_admitted_recovery_handle<Operation, Input, Scope, InputBinding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
        input: &worth_query_declaration::facade::application_operation::ApplicationEncodedInput<
            InputBinding,
        >,
    ) -> Result<WorthQueryRecoveryHandle, Denial>
    where
        InputBinding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                Value = Input,
            >,
        Input: Clone + serde::Serialize + Send + Sync + 'static,
    {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| Denial::new(Kind::CompletionExecutionRequest(cause.into())))?;
        self.settle_recovery_custody(&execution)?;
        let read = self
            .resolve_admitted_application_idempotency(phase, admission, binding)
            .map_err(Denial::original)?;
        let original = match read.into_resolution() {
            WorthQueryApplicationIdempotencyResolution::RecoveredCommitted(original) => original,
            WorthQueryApplicationIdempotencyResolution::IntentDrift => {
                return Err(Denial::new(Kind::GovernedInputMismatch))
            }
            WorthQueryApplicationIdempotencyResolution::ReleasedOriginal(_)
            | WorthQueryApplicationIdempotencyResolution::ProductUnpublished(_)
            | WorthQueryApplicationIdempotencyResolution::ProductPreparationPending(_)
            | WorthQueryApplicationIdempotencyResolution::Unseen => {
                return Err(Denial::new(Kind::RecoveryNotAdmitted))
            }
        };
        let durable = original
            .new_product_head()
            .durable()
            .ok_or_else(|| Denial::new(Kind::ForeignRuntime))?
            .selected_commit();
        let owner = &self.runtime.product_runtime.owner_ref();
        if owner.owner_identity().durable() != Some(durable.owner()) {
            return Err(Denial::new(Kind::ForeignRuntime));
        }
        let material = self
            .runtime
            .primary_provider
            .observe_correction_material(original.commit_reference())
            .map_err(Denial::material)?;
        material
            .admit_recovery_admission(admission, binding, input)
            .map_err(Denial::new)?;
        let publication = match owner.observation_port().readmitted_publication(&durable) {
            Ok(world) => {
                if world.native_receipt() != original.commit_reference()
                    || material.capture.original.ordinal() != world.attempt_identity().ordinal()
                {
                    return Err(Denial::new(Kind::ForeignRuntime));
                }
                let history = owner
                    .inspection_port()
                    .trace_ancestry(
                        world.after().selected_commit().clone(),
                        std::num::NonZeroUsize::MIN,
                    )
                    .map_err(history_read_denial)?;
                let commit = history
                    .commits()
                    .next()
                    .ok_or_else(|| Denial::new(Kind::RecoveryNotAdmitted))?;
                WorthQueryCommittedProductPublication::from_readmitted(&world, commit)
                    .map_err(|cause| Denial::new(Kind::DispatchOwnerReadDenied(cause)))?
            }
            Err(worth_runtime_world::facade::RuntimeWorldServiceDenial::Denied(
                worth_runtime_world::facade::WorldReadmittedPublicationDenial::NotReadmitted,
            )) => {
                let observed = self
                    .runtime
                    .primary_provider
                    .observe_home_publication_for_receipt(
                        &execution,
                        original.commit_reference(),
                        &owner.observation_port(),
                    )
                    .map_err(observed_home_read_denial)?;
                if observed.after().selected_commit() != durable
                    || observed.native_receipt() != original.commit_reference()
                    || material.capture.original.ordinal() != observed.attempt_identity().ordinal()
                {
                    return Err(Denial::new(Kind::ForeignRuntime));
                }
                let signal_publication = match observed.signal_posture() {
                    worth_runtime_world::facade::CompositeComponentChangePosture::RetainExact => {
                        None
                    }
                    worth_runtime_world::facade::CompositeComponentChangePosture::Published => {
                        match owner.inspection_port().trace_ancestry(
                            observed.composite_commit().clone(),
                            std::num::NonZeroUsize::MIN,
                        ) {
                            Ok(history) => {
                                let commit = history
                                    .commits()
                                    .next()
                                    .ok_or_else(|| Denial::new(Kind::RecoveryNotAdmitted))?;
                                if commit.signal_change()
                                    != worth_runtime_world::facade::CompositeComponentChangePosture::Published
                                {
                                    return Err(Denial::new(Kind::RecoveryNotAdmitted));
                                }
                                let signal = commit
                                    .signal_publication_identity()
                                    .cloned()
                                    .ok_or_else(|| Denial::new(Kind::RecoveryNotAdmitted))?;
                                Some(signal)
                            }
                            Err(worth_runtime_world::facade::RuntimeWorldServiceDenial::Denied(
                                worth_runtime_world::facade::CompositeHistoryCatalogDenial::UnknownProtectionTarget(_),
                            )) => None,
                            Err(cause) => return Err(history_read_denial(cause)),
                        }
                    }
                };
                WorthQueryCommittedProductPublication::from_observed_home(
                    &observed,
                    signal_publication,
                )
                .map_err(|cause| Denial::new(Kind::DispatchOwnerReadDenied(cause)))?
            }
            Err(cause) => return Err(world_readmission_denial(cause)),
        };
        let outbox = if let Some(recorded) = original.dispatch_outbox() {
            let observed = self
                .observe_committed_dispatch_outbox_for_correlation(
                    &execution,
                    recorded.correlation(),
                )
                .map_err(|cause| Denial::new(Kind::DispatchOwnerReadDenied(cause)))?;
            if observed.commit_reference() != original.commit_reference()
                || observed.committed_product_publication().composite_commit()
                    != publication.composite_commit()
            {
                return Err(Denial::new(Kind::CorrelationMismatch));
            }
            Some(crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding::from_record_and_ref(
                observed.record().clone(),
                observed.record_ref().clone(),
            ))
        } else {
            None
        };
        let observed = outbox
            .as_ref()
            .and_then(|outbox| {
                self.runtime
                    .dispatch_journal
                    .custody(outbox.record().correlation().digest())
            })
            .and_then(|custody| custody.observation());
        let expiry = match self
            .runtime
            .dispatch_journal
            .recorded_recovery_claim(&durable)
            .map_err(Denial::journal)?
        {
            Some(recorded) if recorded.terminal.is_none() => recorded.claim.expires_at_unix_ms,
            Some(recorded)
                if recorded.terminal
                    != Some(worth_foundational::facade::RecoveryClaimDisposition::Relinquished) =>
            {
                return Err(Denial::new(Kind::RecoveryAlreadyMinted))
            }
            Some(_) | None => {
                crate::domain_computation::application_aftermath::recovery_handle::expiry_deadline(
                    self.runtime.recovery_handles.clock(),
                )?
                .ok_or_else(|| Denial::new(Kind::Expired))?
            }
        };
        let mut binding = WorthQueryRecoveryHandleBinding::from_readmitted(
            self.runtime
                .primary_provider
                .graph_ref()
                .relational_runtime_instance_id(),
            admission,
            publication,
            binding,
            outbox,
            expiry,
            observed,
        )?;
        let claim = WorthQueryRecoveryMintClaim::Home(durable);
        let slot = self.admit_recovery_binding(phase, &mut binding, claim)?;
        Ok(
            crate::domain_computation::application_aftermath::recovery_handle::mint_from_slot(
                binding,
                slot,
                admission.canonical_work(),
                self.runtime.recovery_handles.clone(),
            ),
        )
    }
}

fn observed_home_read_denial(
    cause: crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionReadDenial,
) -> Denial {
    use crate::domain_computation::primary_graph::provider::{
        relational_execution_denial::relational_execution_kind,
        WorthQueryCommittedDispatchOutboxReadDenial as OutboxDenial,
        WorthQueryInboundCompletionReadDenial as ReadDenial,
    };
    match cause {
        ReadDenial::SourceRead(cause) => Denial::new(Kind::DispatchOwnerReadDenied(cause)),
        ReadDenial::ReadRequest(cause) => Denial::new(Kind::DispatchOwnerReadDenied(
            OutboxDenial::ReadExecution(relational_execution_kind(cause, None)),
        )),
        ReadDenial::CommitUnavailable
        | ReadDenial::Malformed
        | ReadDenial::AmbiguousCorrelation
        | ReadDenial::ReconstructionWorkExhausted => Denial::new(Kind::RecoveryNotAdmitted),
    }
}

fn history_read_denial(
    cause: worth_runtime_world::facade::RuntimeWorldServiceDenial<
        worth_runtime_world::facade::CompositeHistoryCatalogDenial,
    >,
) -> Denial {
    use crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxReadDenial as OutboxDenial;
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
    use worth_runtime_world::facade::{
        CompositeHistoryCatalogDenial as HistoryDenial, RuntimeWorldServiceDenial as ServiceDenial,
    };
    match cause {
        ServiceDenial::Denied(HistoryDenial::Cancelled) => Denial::new(
            Kind::DispatchOwnerReadDenied(OutboxDenial::ReadExecution(Err(Stop::Cancelled))),
        ),
        ServiceDenial::Denied(HistoryDenial::DeadlineElapsed) => Denial::new(
            Kind::DispatchOwnerReadDenied(OutboxDenial::ReadExecution(Err(Stop::TimedOut))),
        ),
        _ => Denial::new(Kind::RecoveryNotAdmitted),
    }
}

fn world_readmission_denial(
    cause: worth_runtime_world::facade::RuntimeWorldServiceDenial<
        worth_runtime_world::facade::WorldReadmittedPublicationDenial,
    >,
) -> Denial {
    use crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxReadDenial as OutboxDenial;
    let mapped = crate::domain_computation::primary_graph::provider::committed_dispatch_outbox::recovered::world_denial(cause);
    match mapped {
        OutboxDenial::ReadExecution(_) => Denial::new(Kind::DispatchOwnerReadDenied(mapped)),
        _ => Denial::new(Kind::RecoveryNotAdmitted),
    }
}
