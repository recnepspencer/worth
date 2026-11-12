//! Mint a recovery handle from an admitted commit receipt (R8.28 / R8.62 / R8.33).

use std::sync::Arc;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::managed_run::{
    RecoveryClaimDisposition, WorthQueryRecoveryHandleRegistry, WorthQueryRecoveryMintClaim,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;

use super::binding::WorthQueryRecoveryHandleBinding;
use super::denial::{WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind};
use super::handle::WorthQueryRecoveryHandle;
use super::identity::WorthQueryRecoveryHandleIdentity;

/// Owner-recorded fate of the one recovery claim for a committed operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRecoveryClaimStatus {
    Unclaimed,
    Live,
    Consumed,
    Completed,
    Expired,
    Disposed,
    ForceTerminated,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Mint a recovery handle bound to this runtime's registry and clock.
    ///
    /// A commit receipt is public and a process may publish several application
    /// runtimes over one relational source, so the receipt is the one input a
    /// caller chooses here. It must therefore be *ours*: the receipt carries the
    /// Query runtime authority that admitted its operation, derived at commit
    /// from the admitted operation and not caller-supplied. Without this
    /// comparison a second runtime could mint a handle for a commit it never
    /// made â€” and, because `register_once` claims commits per registry, both
    /// runtimes would hold a live handle for the same commit (R8.28).
    pub fn mint_recovery_handle(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
        ensure_receipt_belongs_to_runtime(
            self.runtime.authority_identity(),
            receipt.runtime_authority(),
        )?;
        self.admit_recovery_custody()?;
        let registry = Arc::clone(&self.runtime.recovery_handles);
        let mut binding = WorthQueryRecoveryHandleBinding::from_receipt(
            receipt,
            expiry_deadline(registry.clock())?,
        )?;
        let slot = self.admit_recovery_binding(phase, &mut binding, claim_for(receipt))?;
        Ok(mint_from_slot(
            binding,
            slot,
            receipt.canonical_work(),
            registry,
        ))
    }

    /// Mint only from the original operation retained through actual failed
    /// cleanup. Recovered data and descriptive cleanup copies open no door.
    pub fn mint_cleanup_recovery_handle(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        cleanup: &crate::domain_computation::primary_graph::WorthQueryApplicationManagedCleanupReceipt,
    ) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
        use crate::domain_computation::primary_graph::WorthQueryCleanupDispatchPosture as Posture;
        match cleanup.posture {
            Posture::Pending | Posture::DiagnosticCopy | Posture::TerminalObserved => {
                return Err(WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::RecoveryNotAdmitted,
                ))
            }
            Posture::NotDeclared | Posture::AttemptedUnresolved | Posture::Observed => {}
        }
        let original = cleanup
            .original
            .as_ref()
            .ok_or_else(|| {
                WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::RecoveryNotAdmitted,
                )
            })?
            .original();
        ensure_receipt_belongs_to_runtime(
            self.runtime.authority_identity(),
            original.original_authority().runtime_authority(),
        )?;
        self.admit_recovery_custody()?;
        let registry = Arc::clone(&self.runtime.recovery_handles);
        let mut binding = WorthQueryRecoveryHandleBinding::from_pending_original(
            original,
            cleanup.dispatch.as_ref(),
            expiry_deadline(registry.clock())?,
        )?;
        let claim = WorthQueryRecoveryMintClaim::new(
            original.original_runtime_instance_id(),
            original.original_publication().composite_commit().clone(),
        );
        let slot = self.admit_recovery_binding(phase, &mut binding, claim)?;
        Ok(mint_from_slot(
            binding,
            slot,
            original.canonical_work(),
            registry,
        ))
    }

    pub(in crate::domain_computation) fn admit_recovery_custody(
        &self,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        self.runtime.product_runtime.admit_directory_capability(
            crate::domain_computation::primary_graph::WorthQueryOperationDurableCapability::RecoveryCustody,
        ).map_err(|cause| WorthQueryRecoveryHandleDenial::new(
            WorthQueryRecoveryHandleDenialKind::DurableStateAdmission(cause),
        ))
    }

    /// Read the exact commit's recovery fate without minting or consuming it.
    pub fn recovery_claim_status(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryRecoveryClaimStatus, WorthQueryRecoveryHandleDenial> {
        ensure_receipt_belongs_to_runtime(
            self.runtime.authority_identity(),
            receipt.runtime_authority(),
        )?;
        let (claimed, terminal) = self
            .runtime
            .recovery_handles
            .claim_state(&claim_for(receipt))
            .map_err(WorthQueryRecoveryHandleDenial::journal)?;
        Ok(status_from_claim_state(claimed, terminal))
    }

    // There is deliberately no `recovery_handle_registry()` here. It existed,
    // was `pub`, and had no caller in either workspace â€” it simply handed any
    // host the whole instance registry without holding a single handle. The
    // registry is reached only through a handle now, and only in fixtures.
}

/// Default handle lifetime when the host does not supply a tighter bound.
pub const DEFAULT_RECOVERY_HANDLE_TTL_MS: u64 = 3_600_000;

pub(in crate::domain_computation) fn mint_from_slot(
    binding: WorthQueryRecoveryHandleBinding,
    slot: crate::domain_computation::managed_run::WorthQueryRecoveryRegistrySlot,
    canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
    registry: Arc<WorthQueryRecoveryHandleRegistry>,
) -> WorthQueryRecoveryHandle {
    WorthQueryRecoveryHandle::new(
        WorthQueryRecoveryHandleIdentity::mint(),
        slot,
        registry,
        binding,
        canonical_work,
    )
}

fn claim_for(receipt: &WorthQueryApplicationCommitReceipt) -> WorthQueryRecoveryMintClaim {
    WorthQueryRecoveryMintClaim::new(
        receipt.provider_runtime_instance_id(),
        receipt
            .committed_product_publication()
            .composite_commit()
            .clone(),
    )
}

/// The minting runtime must be the one that admitted the commit.
pub(crate) fn ensure_receipt_belongs_to_runtime(
    minting: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    committed_by: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
) -> Result<(), WorthQueryRecoveryHandleDenial> {
    if committed_by != minting {
        return Err(WorthQueryRecoveryHandleDenial::new(
            WorthQueryRecoveryHandleDenialKind::ForeignRuntime,
        ));
    }
    Ok(())
}

pub(in crate::domain_computation) fn expiry_deadline(
    clock: &WorthQueryRuntimeClock,
) -> Result<Option<u64>, WorthQueryRecoveryHandleDenial> {
    let sample = clock
        .sample(ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds)
        .map_err(|_| {
            WorthQueryRecoveryHandleDenial::new(WorthQueryRecoveryHandleDenialKind::Expired)
        })?;
    let AspectValue::UInt64(now_ms) = *sample.value() else {
        return Err(WorthQueryRecoveryHandleDenial::new(
            WorthQueryRecoveryHandleDenialKind::Expired,
        ));
    };
    Ok(Some(now_ms.saturating_add(DEFAULT_RECOVERY_HANDLE_TTL_MS)))
}

pub(super) fn status_from_claim_state(
    claimed: bool,
    terminal: Option<RecoveryClaimDisposition>,
) -> WorthQueryRecoveryClaimStatus {
    match terminal {
        Some(RecoveryClaimDisposition::Consumed) => WorthQueryRecoveryClaimStatus::Consumed,
        Some(RecoveryClaimDisposition::Completed) => WorthQueryRecoveryClaimStatus::Completed,
        Some(RecoveryClaimDisposition::Expired) => WorthQueryRecoveryClaimStatus::Expired,
        Some(RecoveryClaimDisposition::Disposed) => WorthQueryRecoveryClaimStatus::Disposed,
        Some(RecoveryClaimDisposition::ForceTerminated) => {
            WorthQueryRecoveryClaimStatus::ForceTerminated
        }
        Some(RecoveryClaimDisposition::Relinquished) => WorthQueryRecoveryClaimStatus::Unclaimed,
        None if claimed => WorthQueryRecoveryClaimStatus::Live,
        None => WorthQueryRecoveryClaimStatus::Unclaimed,
    }
}
