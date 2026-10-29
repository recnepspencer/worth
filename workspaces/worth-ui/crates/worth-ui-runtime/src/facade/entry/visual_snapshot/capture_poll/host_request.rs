pub(super) enum UiHostCaptureAdmission {
    Supported(u64),
    Unsupported,
    AffinityIndeterminate,
}

#[inline(never)]
pub(super) fn capture_byte_limit(
    pixels_required: bool,
    capability: worth_ui_host_contract::UiHostCaptureCapability,
    policy: worth_ui_inspection::UiVisualInspectionPolicy,
) -> UiHostCaptureAdmission {
    match capability {
        worth_ui_host_contract::UiHostCaptureCapability::Unsupported => {
            UiHostCaptureAdmission::Unsupported
        }
        worth_ui_host_contract::UiHostCaptureCapability::GeometryOnly => {
            if pixels_required {
                UiHostCaptureAdmission::Unsupported
            } else {
                UiHostCaptureAdmission::Supported(0)
            }
        }
        worth_ui_host_contract::UiHostCaptureCapability::Pixels {
            maximum_bytes,
            exact_presentation_epoch: true,
        } => UiHostCaptureAdmission::Supported(maximum_bytes.min(policy.maximum_capture_bytes())),
        worth_ui_host_contract::UiHostCaptureCapability::Pixels {
            exact_presentation_epoch: false,
            ..
        } => UiHostCaptureAdmission::AffinityIndeterminate,
    }
}

#[inline(never)]
pub(super) fn host_capture_request(
    pixels_requested: bool,
    capture_identity: u64,
    basis: crate::inspection::visual_snapshot::UiVisualSurfaceCaptureBasis,
    host_session_identity: u64,
    maximum_pixel_bytes: u64,
) -> worth_ui_host_contract::UiHostVisualCaptureRequest {
    worth_ui_host_contract::UiHostVisualCaptureRequest::admitted_by_runtime(
        worth_ui_host_contract::UiHostCaptureRequestIdentity::issued_by_runtime(capture_identity),
        worth_ui_host_contract::UiHostCaptureFrameAffinity::observed_by_runtime(
            basis.frame,
            basis.presentation_attempt,
        ),
        worth_ui_host_contract::UiHostCaptureSurfaceAffinity::observed_by_runtime(
            host_session_identity,
            basis.host_surface,
            basis.binding,
            basis.epoch,
        ),
        worth_ui_host_contract::UiHostCaptureArtifactBudget::admitted_by_runtime(
            pixels_requested && maximum_pixel_bytes > 0,
            maximum_pixel_bytes,
        ),
    )
}
