//! Funded recovery publication and material handoff share the actual Home owners.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::application_aftermath::dispatch_journal::QueryRecoveryClaim;
use crate::domain_computation::application_aftermath::{
    WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial as Denial,
    WorthQueryRecoveryHandleDenialKind as Kind,
};
use crate::domain_computation::managed_run::{
    WorthQueryRecoveryMintClaim, WorthQueryRecoveryRegistrySlot,
};
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableCompositePublicationAttemptIdentity,
};
use worth_query_installation::facade::ApplicationSchema;
mod completion;
mod readmission;
mod settlement;
mod source;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn admit_recovery_binding(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &mut WorthQueryRecoveryHandleBinding,
        claim: WorthQueryRecoveryMintClaim,
    ) -> Result<WorthQueryRecoveryRegistrySlot, Denial> {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| Denial::new(Kind::CompletionExecutionRequest(cause.into())))?;
        self.settle_recovery_custody(&execution)?;
        if !self.runtime.dispatch_journal.is_durable() {
            return self
                .runtime
                .recovery_handles
                .register_once(claim)
                .map_err(|_| Denial::new(Kind::RecoveryAlreadyMinted));
        }
        let publication = binding.committed_product_publication();
        let owner = publication.composite_commit().owner_identity();
        self.runtime
            .product_runtime
            .owner
            .owner_identity()
            .admit_incarnation(owner)
            .map_err(|cause| {
                Denial::new(match cause {
                    worth_proof::HomeIncarnationDenial::ForeignHome => Kind::ForeignRuntime,
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => Kind::StaleIncarnation,
                })
            })?;
        let owner = owner
            .durable()
            .ok_or_else(|| Denial::new(Kind::ForeignRuntime))?;
        let original = DurableCompositeCommitIdentity::from_recorded(
            owner,
            publication.composite_commit().ordinal(),
        );
        let attempt = DurableCompositePublicationAttemptIdentity::from_recorded(
            owner,
            publication.publication_attempt().ordinal(),
        );
        let material = self
            .runtime
            .primary_provider
            .observe_correction_material(binding.commit_reference())
            .map_err(Denial::material)?;
        if material.capture.original != attempt
            || &material.ready.commit != binding.commit_reference()
        {
            return Err(Denial::new(Kind::GovernedInputMismatch));
        }
        let admission = self
            .runtime
            .dispatch_journal
            .begin_recovery_admission(&execution)
            .map_err(Denial::journal)?;
        let recorded = match admission.claim(&original).map_err(Denial::journal)? {
            Some(recorded) if recorded.terminal.is_none() => {
                if recorded.claim.attempt != attempt {
                    return Err(Denial::new(Kind::ForeignRuntime));
                }
                recorded
            }
            Some(recorded)
                if recorded.terminal
                    != Some(worth_foundational::facade::RecoveryClaimDisposition::Relinquished) =>
            {
                return Err(Denial::new(Kind::RecoveryAlreadyMinted))
            }
            Some(_) | None => {
                let pin = self
                    .runtime
                    .primary_provider
                    .acquire_recovery_material(attempt, &execution)
                    .map_err(Denial::material)?;
                admission
                    .acquire(
                        QueryRecoveryClaim {
                            original,
                            attempt,
                            material_pin: pin,
                            expires_at_unix_ms: binding
                                .expires_at_unix_ms()
                                .ok_or_else(|| Denial::new(Kind::Expired))?,
                        },
                        &execution,
                    )
                    .map_err(Denial::journal)?
            }
        };
        binding.readmit_expiry(recorded.claim.expires_at_unix_ms);
        self.runtime
            .recovery_handles
            .register_durable_once(claim, recorded)
            .map_err(|_| Denial::new(Kind::RecoveryAlreadyMinted))
    }
    /// Settle framework-retained Drop and acknowledged terminal material work.
    pub fn maintain_recovery_claims(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), Denial> {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| Denial::new(Kind::CompletionExecutionRequest(cause.into())))?;
        self.settle_recovery_custody(&execution)
    }
}
