//! Staged text coverage reading and the never-settling port the certification drives.
use super::{presentation, Denial, UiNativeFinalizedTextForeground, UiNativeRetainedDrawList};

pub(super) fn coverage(
    retained: &UiNativeRetainedDrawList,
) -> Result<Vec<UiNativeFinalizedTextForeground>, Denial> {
    let (_, appearance) = retained
        .staged_appearance
        .as_ref()
        .ok_or(Denial::CommandMismatch)?;
    use crate::native::presentation::appearance::UiNativeAppearanceCommand as Command;
    appearance
        .ordered_keys()
        .iter()
        .filter_map(|key| match appearance.command(*key) {
            Some(Command::TextForeground(value)) => Some(Ok(value.clone())),
            // The staged overlay order is the surface stack, not text coverage.
            Some(Command::OverlayOrder(_)) => None,
            _ => Some(Err(Denial::CommandMismatch)),
        })
        .collect()
}

pub(super) struct UnsettledPort;
impl presentation::UiNativePendingExternalObligation for UnsettledPort {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending)
    }
}
