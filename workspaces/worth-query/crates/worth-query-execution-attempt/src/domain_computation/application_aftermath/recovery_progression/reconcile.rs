//! Reconcile transition — keys RuntimeWithExternalOwner authority (R8.30).

use worth_query_installation::facade::InstalledCorrectionAuthority;

use crate::domain_computation::managed_run::RecoveryClaimDisposition;

use super::super::recovery_handle::{
    RelinquishOnDenial, WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBinding,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
use super::authority::{require_fresh_effect_authority, WorthQueryRecoveryEffectAuthority};

/// Proof that reconcile was admitted against the installed authority axis.
#[derive(Debug)]
pub struct WorthQueryRecoveryReconcileAdmission {
    binding: WorthQueryRecoveryHandleBinding,
}

impl WorthQueryRecoveryReconcileAdmission {
    pub const fn binding(&self) -> &WorthQueryRecoveryHandleBinding {
        &self.binding
    }
}

/// Admits reconciliation for a recovery handle and consumes the handle.
///
/// `authority` must be fresh effect authority minted for this exact handle, and
/// the installed aftermath must name an external owner as a correction
/// authority. The returned admission is proof to proceed; this call changes
/// nothing itself. On denial the handle is relinquished, not consumed.
pub fn reconcile_recovery_handle(
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    handle: WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryEffectAuthority,
) -> Result<
    WorthQueryRecoveryReconcileAdmission,
    super::super::recovery_handle::WorthQueryRecoveryTransitionDenial,
> {
    let handle = handle.admit(|handle| {
        require_fresh_effect_authority(handle, authority)?;
        match handle.binding().installed_aftermath().authority() {
            InstalledCorrectionAuthority::RuntimeWithExternalOwner => Ok(()),
            InstalledCorrectionAuthority::RuntimeAlone
            | InstalledCorrectionAuthority::NotCorrectable => {
                Err(WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::ReconciliationNotAdmitted,
                ))
            }
        }
    })?;
    Ok(WorthQueryRecoveryReconcileAdmission {
        binding: handle.consume(phase, RecoveryClaimDisposition::Consumed)?,
    })
}
