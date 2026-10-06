#[derive(Clone)]
pub(super) struct UiNativeCaptureSource {
    host_session: u64,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    epoch: worth_ui_host_contract::UiHostPresentationEpoch,
    transform: Option<worth_ui_host_contract::UiHostCoordinateTransform>,
    regions: Box<[worth_ui_host_contract::UiHostRealizedRegion]>,
}

pub(super) struct UiNativeCaptureSourceInput {
    pub(super) host_session: u64,
    pub(super) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(super) attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    pub(super) host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    pub(super) binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    pub(super) epoch: worth_ui_host_contract::UiHostPresentationEpoch,
    pub(super) transform: Option<worth_ui_host_contract::UiHostCoordinateTransform>,
    pub(super) regions: Vec<worth_ui_host_contract::UiHostRealizedRegion>,
}

impl UiNativeCaptureSource {
    pub(super) fn completed(input: UiNativeCaptureSourceInput) -> Self {
        Self {
            host_session: input.host_session,
            frame: input.frame,
            attempt: input.attempt,
            host_surface: input.host_surface,
            binding: input.binding,
            epoch: input.epoch,
            transform: input.transform,
            regions: input.regions.into_boxed_slice(),
        }
    }

    pub(super) fn matches(
        &self,
        request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    ) -> bool {
        self.host_session == request.host_session_identity()
            && self.frame == request.frame()
            && self.attempt == request.presentation_attempt()
            && self.host_surface == request.host_surface()
            && self.binding == request.binding()
            && self.epoch == request.expected_epoch()
    }

    pub(super) const fn transform(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostCoordinateTransform> {
        self.transform
    }

    pub(super) fn regions(&self) -> Vec<worth_ui_host_contract::UiHostRealizedRegion> {
        self.regions.to_vec()
    }

    pub(super) const fn epoch(&self) -> worth_ui_host_contract::UiHostPresentationEpoch {
        self.epoch
    }
}

pub(super) fn coordinate_transform(
    window: &crate::native::event_loop::UiNativeOwnedWindow,
    graphics: &crate::native::UiNativePresentationAccess,
) -> Option<worth_ui_host_contract::UiHostCoordinateTransform> {
    let origin = window.inner_position()?;
    let physical = graphics.extent();
    let scale = graphics.scale_factor() as f32;
    if physical.contains(&0) || !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let logical = [physical[0] as f32 / scale, physical[1] as f32 / scale];
    Some(
        worth_ui_host_contract::UiHostCoordinateTransform::observed_by_host(
            worth_ui_host_contract::UiHostClientAreaObservation::observed_by_host(
                [origin.x, origin.y],
                physical,
            ),
            worth_ui_host_contract::UiHostViewportTransformObservation::observed_by_host(
                logical,
                [scale, scale],
                [0.0, 0.0],
            ),
            worth_ui_host_contract::UiHostCoordinatePosture::observed_by_host(
                worth_ui_host_contract::UiHostCoordinateOrientation::TopLeftOrigin,
                worth_ui_host_contract::UiHostCoordinateRounding::PixelCenterNearest,
            ),
        ),
    )
}
