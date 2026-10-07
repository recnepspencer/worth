use super::WorthQueryRelationalSourceOwner;
use crate::domain_computation::primary_graph::WorthQueryApplicationCloseDenial;
use worth_proof::TransitionOutcome;
use worth_relational::facade::runtime::RelationalRuntimeAdmissionHoldDenial;

impl WorthQueryRelationalSourceOwner {
    pub(in crate::domain_computation) fn close_with_image<T>(
        &self,
        assemble: impl FnOnce(
            worth_relational::facade::durability::RelationalNativeCheckpoint,
        )
            -> Result<T, worth_relational::facade::durability::DurabilityError>,
    ) -> Result<T, WorthQueryApplicationCloseDenial> {
        let mut runtime = match self.runtime.try_lock() {
            Ok(runtime) => runtime,
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err(WorthQueryApplicationCloseDenial::AdmissionsActive);
            }
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
        };
        let hold = match runtime.try_hold_admission() {
            TransitionOutcome::Success(hold) => hold,
            TransitionOutcome::Denied(RelationalRuntimeAdmissionHoldDenial::AdmissionsActive) => {
                return Err(WorthQueryApplicationCloseDenial::AdmissionsActive);
            }
            TransitionOutcome::Denied(denial) => {
                return Err(WorthQueryApplicationCloseDenial::Owner(denial));
            }
        };
        let native = hold
            .native_checkpoint()
            .and_then(|image| {
                #[cfg(test)]
                if self
                    .fail_next_closing_capture
                    .swap(false, std::sync::atomic::Ordering::Relaxed)
                {
                    return Err(worth_relational::facade::durability::DurabilityError::new(
                    worth_relational::facade::durability::RecoveryFailureClass::CorruptCheckpoint,
                    "injected closing checkpoint capture failure",
                ));
                }
                Ok(image)
            })
            .map_err(WorthQueryApplicationCloseDenial::Capture)?;
        let image = assemble(native).map_err(WorthQueryApplicationCloseDenial::Capture)?;
        hold.seal();
        Ok(image)
    }
}
