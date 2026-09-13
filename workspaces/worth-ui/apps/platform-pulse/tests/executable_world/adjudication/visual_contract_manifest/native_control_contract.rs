use super::{
    checked_in, model::PlatformPulseVisualContractManifest,
    validation::PlatformPulseVisualContractFailure,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::adjudication) struct PlatformPulseNativeControlContract {
    logical_client_extent: [u32; 2],
    target_rect: [u32; 4],
    label_rect: [u32; 4],
    background_rgba: [u8; 4],
    channel_tolerance: u8,
}

pub(in crate::adjudication) fn action_control(
) -> Result<PlatformPulseNativeControlContract, PlatformPulseVisualContractFailure> {
    control("platform.pulse.target.run_live_action")
}

pub(in crate::adjudication) fn portal_control(
) -> Result<PlatformPulseNativeControlContract, PlatformPulseVisualContractFailure> {
    control("platform.pulse.target.open_portal")
}

pub(in crate::adjudication) fn portal_control_for_extent(
    logical_client_extent: [u32; 2],
) -> Result<PlatformPulseNativeControlContract, PlatformPulseVisualContractFailure> {
    control_for_extent("platform.pulse.target.open_portal", logical_client_extent)
}

pub(in crate::adjudication) fn confirmation_control(
) -> Result<PlatformPulseNativeControlContract, PlatformPulseVisualContractFailure> {
    control("platform.pulse.target.confirm_live_action")
}

fn control(
    identity: &str,
) -> Result<PlatformPulseNativeControlContract, PlatformPulseVisualContractFailure> {
    control_for_extent(identity, [960, 600])
}

fn control_for_extent(
    identity: &str,
    logical_client_extent: [u32; 2],
) -> Result<PlatformPulseNativeControlContract, PlatformPulseVisualContractFailure> {
    let manifest = checked_in()?;
    from_manifest(&manifest, identity, logical_client_extent)
}

fn from_manifest(
    manifest: &PlatformPulseVisualContractManifest,
    identity: &str,
    logical_client_extent: [u32; 2],
) -> Result<PlatformPulseNativeControlContract, PlatformPulseVisualContractFailure> {
    let layout = manifest
        .layouts
        .iter()
        .find(|layout| layout.logical_client_extent == logical_client_extent)
        .ok_or(PlatformPulseVisualContractFailure::Target)?;
    let target = layout
        .minimum_targets
        .iter()
        .find(|target| target.identity == identity)
        .ok_or(PlatformPulseVisualContractFailure::Target)?;
    let label = layout
        .text_bounds
        .iter()
        .find(|text| text.identity == target.label_identity)
        .ok_or(PlatformPulseVisualContractFailure::TextContainment)?;
    let background_rgba = manifest
        .tokens
        .iter()
        .find(|token| token.role == target.token_role)
        .map(|token| token.rgba)
        .ok_or(PlatformPulseVisualContractFailure::Token)?;
    Ok(PlatformPulseNativeControlContract {
        logical_client_extent: layout.logical_client_extent,
        target_rect: target.rect,
        label_rect: label.rect,
        background_rgba,
        channel_tolerance: manifest.limits.channel_tolerance,
    })
}

impl PlatformPulseNativeControlContract {
    pub(in crate::adjudication) const fn logical_client_extent(self) -> [u32; 2] {
        self.logical_client_extent
    }

    pub(in crate::adjudication) const fn target_rect(self) -> [u32; 4] {
        self.target_rect
    }

    pub(in crate::adjudication) const fn label_rect(self) -> [u32; 4] {
        self.label_rect
    }

    pub(in crate::adjudication) const fn background_rgba(self) -> [u8; 4] {
        self.background_rgba
    }

    pub(in crate::adjudication) const fn channel_tolerance(self) -> u8 {
        self.channel_tolerance
    }

    pub(in crate::adjudication) const fn preferred_logical_point(self) -> [u32; 2] {
        [self.target_rect[0] + 8, self.target_rect[1] + 8]
    }
}
