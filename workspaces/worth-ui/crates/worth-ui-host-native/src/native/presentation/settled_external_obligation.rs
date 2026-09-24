use super::UiNativePendingExternalObligation;

/// The external work of a presentation whose physical request already
/// settled. A recovery admitted after that settlement waits on work that is
/// finished, so it completes on its first poll.
pub(crate) struct UiNativeSettledExternalObligation;

impl UiNativePendingExternalObligation for UiNativeSettledExternalObligation {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed)
    }
}
