//! Dispose transition — terminal, consumes the handle (R8.29 / R8.30).

use crate::domain_computation::managed_run::WorthQueryRecoveryResourceTerminal;

use super::super::recovery_handle::{
    RelinquishOnDenial, WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBinding,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
use super::authority::{require_fresh_effect_authority, WorthQueryRecoveryEffectAuthority};

/// Proof that dispose completed and the handle is gone.
#[derive(Debug)]
pub struct WorthQueryRecoveryDisposalReceipt {
    binding: WorthQueryRecoveryHandleBinding,
}

impl WorthQueryRecoveryDisposalReceipt {
    pub const fn binding(&self) -> &WorthQueryRecoveryHandleBinding {
        &self.binding
    }
}

/// Disposes of a recovery handle, ending the custody it stands for.
///
/// `authority` must be fresh effect authority minted for this exact handle. On
/// success the handle is consumed as disposed and the receipt names its binding.
/// On denial the handle is relinquished, not disposed.
pub fn dispose_recovery_handle(
    handle: WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryEffectAuthority,
) -> Result<WorthQueryRecoveryDisposalReceipt, WorthQueryRecoveryHandleDenial> {
    let handle = handle.admit(|handle| require_fresh_effect_authority(handle, authority))?;
    Ok(WorthQueryRecoveryDisposalReceipt {
        binding: handle.consume(WorthQueryRecoveryResourceTerminal::Disposed)?,
    })
}

/// Expire after `evaluate_expiry` proves the deadline. Records the sample via
/// the decision argument (R8.7 M3).
pub fn expire_recovery_handle(
    handle: WorthQueryRecoveryHandle,
    decision: &super::WorthQueryRecoveryExpiryDecision,
) -> Result<WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial> {
    let handle = handle.admit(|handle| {
        if decision.handle_authority() == handle.authority_identity() {
            Ok(())
        } else {
            Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied,
            ))
        }
    })?;
    // `WorthQueryRecoveryExpiryDecision` is the expired branch of a typed
    // runtime-clock evaluation. Current evidence has a different type and
    // cannot reach this signature (R8.7 M2 / R8.31).
    handle.consume(WorthQueryRecoveryResourceTerminal::Expired)
}
