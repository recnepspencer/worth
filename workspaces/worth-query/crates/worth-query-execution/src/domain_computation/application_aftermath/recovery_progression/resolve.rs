//! Resolve via admitted idempotency read — inherited G5 taxonomy (R8.32).
//!
//! The resolution value is privately minted by the admitted graph read. Callers
//! cannot construct a taxonomy answer and feed it to resolve. The read also
//! carries the idempotency binding it was read for, so a foreign read cannot
//! answer for a different handle.

use crate::domain_computation::application_aftermath::WorthQueryExternalDispatchPostureKind;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
};

use super::super::recovery_handle::{
    RelinquishOnDenial, WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial,
    WorthQueryRecoveryHandleDenialKind,
};
use super::authority::{require_fresh_effect_authority, WorthQueryRecoveryEffectAuthority};
use crate::domain_computation::managed_run::WorthQueryRecoveryResourceTerminal;

/// Proof that an admitted graph idempotency read produced this resolution for
/// a specific binding.
///
/// Constructed only by
/// [`resolve_admitted_application_idempotency`](crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime::resolve_admitted_application_idempotency).
/// Not `Clone`: a one-shot read result must not be replayable indefinitely.
#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedIdempotencyRead {
    binding: WorthQueryApplicationIdempotencyBinding,
    resolution: WorthQueryApplicationIdempotencyResolution,
    _private: (),
}

impl WorthQueryAdmittedIdempotencyRead {
    pub(crate) const fn mint(
        binding: WorthQueryApplicationIdempotencyBinding,
        resolution: WorthQueryApplicationIdempotencyResolution,
    ) -> Self {
        Self {
            binding,
            resolution,
            _private: (),
        }
    }

    pub const fn binding(&self) -> WorthQueryApplicationIdempotencyBinding {
        self.binding
    }

    pub const fn resolution(&self) -> &WorthQueryApplicationIdempotencyResolution {
        &self.resolution
    }

    pub fn into_resolution(self) -> WorthQueryApplicationIdempotencyResolution {
        self.resolution
    }
}

/// Resolve consumes the handle and returns the inherited resolution taxonomy.
pub fn resolve_recovery_handle(
    handle: WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryEffectAuthority,
    admitted_read: WorthQueryAdmittedIdempotencyRead,
) -> Result<WorthQueryApplicationIdempotencyResolution, WorthQueryRecoveryHandleDenial> {
    let handle = handle.admit(|handle| {
        require_fresh_effect_authority(handle, authority)?;
        if admitted_read.binding() != handle.binding().idempotency() {
            // Handle is right; the read is foreign. Nothing was resolved, so
            // this relinquishes rather than terminates — the caller can pair
            // the same commit's recovery with the right read (Q8.21-L11).
            return Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::ForeignIdempotencyRead,
            ));
        }
        Ok(())
    })?;
    if handle
        .binding()
        .provider_posture()
        .is_some_and(|posture| posture.kind() == WorthQueryExternalDispatchPostureKind::Unresolved)
    {
        // Unresolved external posture stays unresolved — resolve must not
        // upgrade it. This one *consumes* rather than relinquishes on purpose:
        // authority, handle, and read were all correct, so resolve really ran.
        // It simply has no resolution to report. Retrying it would ask the same
        // question of the same unresolved posture (Q8.21-L11).
        let _ = handle.consume(WorthQueryRecoveryResourceTerminal::Consumed)?;
        return Err(WorthQueryRecoveryHandleDenial::new(
            WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture,
        ));
    }
    let _ = handle.consume(WorthQueryRecoveryResourceTerminal::Consumed)?;
    Ok(admitted_read.into_resolution())
}
