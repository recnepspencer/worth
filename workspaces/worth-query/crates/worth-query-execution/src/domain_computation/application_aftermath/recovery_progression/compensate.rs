//! Compensate transition — keys installed Compensation mechanism (R8.30).

use worth_query_installation::facade::InstalledCorrectionMechanism;

use crate::domain_computation::managed_run::WorthQueryRecoveryResourceTerminal;

use super::super::recovery_handle::{
    RelinquishOnDenial, WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBinding,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
use super::authority::{require_fresh_effect_authority, WorthQueryRecoveryEffectAuthority};

/// Proof that compensate was admitted against the installed mechanism axis.
#[derive(Debug)]
pub struct WorthQueryRecoveryCompensateAdmission {
    binding: WorthQueryRecoveryHandleBinding,
}

impl WorthQueryRecoveryCompensateAdmission {
    pub const fn binding(&self) -> &WorthQueryRecoveryHandleBinding {
        &self.binding
    }
}

/// Admits compensation for a recovery handle and consumes the handle.
///
/// `authority` must be fresh effect authority minted for this exact handle, and
/// the operation's installed aftermath must declare a compensation mechanism.
/// The returned admission is proof to proceed; this call performs no
/// compensating change itself. On denial the handle is relinquished, not
/// consumed.
pub fn compensate_recovery_handle(
    handle: WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryEffectAuthority,
) -> Result<WorthQueryRecoveryCompensateAdmission, WorthQueryRecoveryHandleDenial> {
    let handle = handle.admit(|handle| {
        require_fresh_effect_authority(handle, authority)?;
        match handle.binding().installed_aftermath().mechanism() {
            Some(InstalledCorrectionMechanism::Compensation(_)) => Ok(()),
            Some(InstalledCorrectionMechanism::RecordedInverse(_)) | None => {
                Err(WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::CompensationNotAdmitted,
                ))
            }
        }
    })?;
    Ok(WorthQueryRecoveryCompensateAdmission {
        binding: handle.consume(WorthQueryRecoveryResourceTerminal::Consumed)?,
    })
}
