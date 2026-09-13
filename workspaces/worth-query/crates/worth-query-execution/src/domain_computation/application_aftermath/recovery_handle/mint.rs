//! Mint a recovery handle from an admitted commit receipt (R8.28 / R8.62 / R8.33).

use std::sync::Arc;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::managed_run::{
    WorthQueryRecoveryHandleRegistry, WorthQueryRecoveryMintClaim,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;

use super::binding::WorthQueryRecoveryHandleBinding;
use super::denial::{WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind};
use super::handle::WorthQueryRecoveryHandle;
use super::identity::WorthQueryRecoveryHandleIdentity;

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
    /// made — and, because `register_once` claims commits per registry, both
    /// runtimes would hold a live handle for the same commit (R8.28).
    pub fn mint_recovery_handle(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
        ensure_receipt_belongs_to_runtime(
            self.runtime.authority_identity(),
            receipt.runtime_authority(),
        )?;
        mint_recovery_handle_in_registry(receipt, Arc::clone(&self.recovery_handles))
    }

    // There is deliberately no `recovery_handle_registry()` here. It existed,
    // was `pub`, and had no caller in either workspace — it simply handed any
    // host the whole instance registry without holding a single handle. The
    // registry is reached only through a handle now, and only in fixtures.
}

/// Default handle lifetime when the host does not supply a tighter bound.
pub const DEFAULT_RECOVERY_HANDLE_TTL_MS: u64 = 3_600_000;

/// Mint one framework-owned recovery handle into the runtime's registry.
///
/// Derives exactly one identity (R8.33). Binding fields come from the receipt
/// (R8.62) — never from caller assertions. Lookup/inspection remain at zero
/// basis/digest cost after mint.
///
/// The deadline comes from the registry's own runtime clock. Passing the clock
/// in used to be optional, and the `None` arm minted a handle that never
/// expired; there is now no lane through which the deadline can be omitted or
/// sourced from a clock other than the one the owning runtime was published
/// with (R8.31).
pub(crate) fn mint_recovery_handle_in_registry(
    receipt: &WorthQueryApplicationCommitReceipt,
    registry: Arc<WorthQueryRecoveryHandleRegistry>,
) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
    let expires_at = expiry_deadline(registry.clock())?;
    let binding = WorthQueryRecoveryHandleBinding::from_receipt(receipt, expires_at)?;
    let claim = WorthQueryRecoveryMintClaim::new(
        receipt.provider_runtime_instance_id(),
        receipt
            .committed_product_publication()
            .composite_commit()
            .clone(),
    );
    let slot = registry.register_once(claim).map_err(|_| {
        WorthQueryRecoveryHandleDenial::new(
            WorthQueryRecoveryHandleDenialKind::RecoveryAlreadyMinted,
        )
    })?;
    // Exactly one identity for the successfully claimed handle. A rejected
    // receipt clone performs no identity mint or canonical derivation work.
    let identity = WorthQueryRecoveryHandleIdentity::mint();
    Ok(WorthQueryRecoveryHandle::new(
        identity,
        slot,
        registry,
        binding,
        receipt.canonical_work(),
    ))
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

fn expiry_deadline(
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
