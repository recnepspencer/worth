//! Fresh admission joins actual World/Native originals and retained original input.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution, WorthQueryCommittedProductPublication,
};

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
        self.admit_recovery_custody()?;
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
        let owner = &self.runtime.product_runtime.owner;
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
        let world = owner
            .observation_port()
            .readmitted_publication(&durable)
            .map_err(|_| Denial::new(Kind::RecoveryNotAdmitted))?;
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
            .map_err(|_| Denial::new(Kind::RecoveryNotAdmitted))?;
        let commit = history
            .commits()
            .next()
            .ok_or_else(|| Denial::new(Kind::RecoveryNotAdmitted))?;
        let publication = WorthQueryCommittedProductPublication::from_readmitted(&world, commit)
            .map_err(|cause| Denial::new(Kind::DispatchOwnerReadDenied(cause)))?;
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
            Some(crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding::from_observed_original(&observed))
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
                .graph
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
