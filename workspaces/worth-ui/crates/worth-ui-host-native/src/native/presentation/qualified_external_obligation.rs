use super::{port, UiNativePendingExternalObligation};

pub(crate) struct UiNativeQualifiedExternalObligation {
    inner: Box<dyn UiNativePendingExternalObligation>,
    effects_indeterminate: QualifiedObservation,
    duplicate_completed: QualifiedObservation,
}

/// A qualified observation the plan armed is reported exactly once.
enum QualifiedObservation {
    Unarmed,
    Armed,
    Reported,
}

impl QualifiedObservation {
    const fn armed_if(armed: bool) -> Self {
        if armed {
            Self::Armed
        } else {
            Self::Unarmed
        }
    }

    fn report(&mut self) -> bool {
        let armed = matches!(self, Self::Armed);
        if armed {
            *self = Self::Reported;
        }
        armed
    }
}

impl UiNativeQualifiedExternalObligation {
    pub(crate) fn new(
        inner: Box<dyn UiNativePendingExternalObligation>,
        effects_indeterminate: bool,
        duplicate_completed: bool,
    ) -> Self {
        Self {
            inner,
            effects_indeterminate: QualifiedObservation::armed_if(effects_indeterminate),
            duplicate_completed: QualifiedObservation::armed_if(duplicate_completed),
        }
    }
}

impl UiNativePendingExternalObligation for UiNativeQualifiedExternalObligation {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        if self.effects_indeterminate.report() {
            return basis.observe_qualified_external(
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::EffectsIndeterminate,
            );
        }
        self.inner.poll_observation(basis, device)
    }

    fn take_presented_observation(&mut self) -> Option<port::UiNativePresentationPortObservation> {
        self.inner.take_presented_observation()
    }

    fn take_duplicate_completed_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
    ) -> Option<crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation>
    {
        if !self.duplicate_completed.report() {
            return None;
        }
        Some(basis.observe_qualified_external(
            crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed,
        ))
    }
}
