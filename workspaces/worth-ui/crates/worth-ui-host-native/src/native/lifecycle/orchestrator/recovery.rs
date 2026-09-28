use crate::native::lifecycle::recovery::UiNativePhysicalRecoveryPreparation;
#[cfg(feature = "certification-support")]
use crate::native::presentation::UiNativePresentationFault;
use crate::native::presentation::UiNativePresentationRecoveryClass;
#[cfg(feature = "certification-support")]
use crate::native::presentation::{
    UiNativeSurfaceAcquireFailure, UiNativeSurfaceFailureDisposition,
};
use crate::native::{UiNativeRecoveryCause, UiNativeRecoveryLineage, UiNativeRecoveryRequirement};

use super::{
    UiNativeLifecycleDirective, UiNativeLifecycleOrchestrator, UiNativeSurfaceBasisTransition,
};

impl UiNativeLifecycleOrchestrator {
    pub(crate) fn observe_surface_transition(
        &mut self,
        transition: UiNativeSurfaceBasisTransition,
        bindings: impl IntoIterator<Item = u64>,
    ) -> UiNativeLifecycleDirective {
        let (cause, directive) = match transition {
            UiNativeSurfaceBasisTransition::ZeroSized
            | UiNativeSurfaceBasisTransition::Minimized => (
                UiNativeRecoveryCause::Resize,
                UiNativeLifecycleDirective::WaitForVisibility,
            ),
            // An ordinary resize keeps every retained resource: the next
            // presentation repaints the successor target, nothing reconstructs.
            UiNativeSurfaceBasisTransition::Resize => return UiNativeLifecycleDirective::Repaint,
            UiNativeSurfaceBasisTransition::Dpi => (
                UiNativeRecoveryCause::Dpi,
                UiNativeLifecycleDirective::Reconstruct(UiNativePresentationRecoveryClass::Dpi),
            ),
        };
        self.require_recovery_for(bindings, cause);
        directive
    }

    #[cfg(feature = "certification-support")]
    pub(crate) fn observe_protocol_fault(
        &mut self,
        fault: UiNativePresentationFault,
        bindings: impl IntoIterator<Item = u64>,
    ) -> UiNativeLifecycleDirective {
        let failure = match fault {
            UiNativePresentationFault::Timeout => UiNativeSurfaceAcquireFailure::Timeout,
            UiNativePresentationFault::Occluded => UiNativeSurfaceAcquireFailure::Occluded,
            UiNativePresentationFault::Outdated => UiNativeSurfaceAcquireFailure::Outdated,
            UiNativePresentationFault::SurfaceLost => UiNativeSurfaceAcquireFailure::Lost,
            UiNativePresentationFault::Validation => UiNativeSurfaceAcquireFailure::Validation,
            UiNativePresentationFault::DeviceLost => UiNativeSurfaceAcquireFailure::DeviceLost,
        };
        match Self::classify_surface_failure(failure) {
            UiNativeSurfaceFailureDisposition::RetryAfterTimeout => {
                UiNativeLifecycleDirective::RetryAfterTimeout
            }
            UiNativeSurfaceFailureDisposition::WaitForVisibility => {
                UiNativeLifecycleDirective::WaitForVisibility
            }
            UiNativeSurfaceFailureDisposition::ValidationRejected => {
                UiNativeLifecycleDirective::RejectValidation
            }
            UiNativeSurfaceFailureDisposition::ReconstructionRequired(recovery) => {
                let cause = recovery.cause();
                self.require_recovery_for(bindings, cause);
                UiNativeLifecycleDirective::Reconstruct(recovery_class(cause))
            }
        }
    }

    pub(crate) fn require_recovery_for(
        &mut self,
        bindings: impl IntoIterator<Item = u64>,
        cause: UiNativeRecoveryCause,
    ) {
        for binding in bindings {
            self.recovery.require(binding, cause);
        }
    }

    pub(crate) fn require_recovery(&mut self, binding: u64, cause: UiNativeRecoveryCause) {
        self.recovery.require(binding, cause);
    }

    pub(crate) fn recovery_required(&self, binding: u64) -> bool {
        self.recovery.requires(binding)
    }

    pub(crate) fn recovery_ready(&self, binding: u64) -> bool {
        self.recovery.ready(binding)
    }

    pub(crate) fn physical_recovery_preparation(
        &self,
        binding: u64,
    ) -> Option<UiNativePhysicalRecoveryPreparation> {
        self.recovery.physical_preparation(binding)
    }

    pub(crate) fn commit_physical_recovery(
        &mut self,
        preparation: UiNativePhysicalRecoveryPreparation,
        device_generation: u64,
        surface_generation: u64,
    ) -> bool {
        self.recovery
            .commit_physical(preparation, device_generation, surface_generation)
    }

    pub(crate) fn take_recovery(&mut self, binding: u64) -> Option<UiNativeRecoveryRequirement> {
        self.recovery.take(binding)
    }

    pub(crate) fn settle_recovery(&mut self, recovery: UiNativeRecoveryRequirement) -> bool {
        self.recovery.settle(recovery)
    }

    pub(crate) fn restore_recovery(&mut self, recovery: UiNativeRecoveryRequirement) {
        self.recovery.restore(recovery);
    }

    pub(crate) fn resolve_recovery(&mut self, binding: u64) -> bool {
        self.recovery.resolve(binding)
    }

    pub(crate) fn transfer_recovery(&mut self, predecessor: u64, successor: u64) -> bool {
        self.recovery.transfer(predecessor, successor)
    }

    pub(crate) fn park_recovery(&mut self, binding: u64, lineage: UiNativeRecoveryLineage) -> bool {
        self.recovery.park(binding, lineage)
    }

    pub(crate) fn claim_recovery(
        &mut self,
        lineage: UiNativeRecoveryLineage,
        successor: u64,
    ) -> bool {
        self.recovery.claim(lineage, successor)
    }

    pub(crate) fn clear_recovery(&mut self) {
        self.recovery.clear();
    }

    pub(crate) fn recovery_count(&self) -> usize {
        self.recovery.len()
    }
}

#[cfg(feature = "certification-support")]
const fn recovery_class(cause: UiNativeRecoveryCause) -> UiNativePresentationRecoveryClass {
    match cause {
        UiNativeRecoveryCause::SurfaceOutdated => {
            UiNativePresentationRecoveryClass::SurfaceOutdated
        }
        UiNativeRecoveryCause::SurfaceLost => UiNativePresentationRecoveryClass::SurfaceLost,
        UiNativeRecoveryCause::DeviceLost => UiNativePresentationRecoveryClass::DeviceLost,
        UiNativeRecoveryCause::PresentationIndeterminate => {
            UiNativePresentationRecoveryClass::PresentationIndeterminate
        }
        UiNativeRecoveryCause::Resize => UiNativePresentationRecoveryClass::Resize,
        UiNativeRecoveryCause::Dpi => UiNativePresentationRecoveryClass::Dpi,
        UiNativeRecoveryCause::DerivedStateLost => {
            UiNativePresentationRecoveryClass::PresentationIndeterminate
        }
    }
}
