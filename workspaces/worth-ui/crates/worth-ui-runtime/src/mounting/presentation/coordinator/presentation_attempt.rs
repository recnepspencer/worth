use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedPresentationAttemptIdentity, UiPresentationDeadline,
    WorthUiHostKind,
};

use super::super::consumption_view::{
    UiMountedHostPresentationAuthority, UiRuntimeMountedFrameConsumptionInput,
};
use super::super::outcome::{
    UiMountedSurfacePresentationReceipt, UiMountedSurfacePresentationRejection,
};
use super::super::terminal::UiIndeterminatePresentationEvidence;
use crate::facade::UiHostEffectPort;
use crate::native_platform::text_presentation::UiNativeTextPresentationReadiness;

pub(super) struct UiMountedPresentationStart<'host, 'authority> {
    pub(super) frame: crate::mounting::UiPreparedMountedFrame,
    pub(super) retention: crate::mounting::retention::UiMountedRetentionReservation,
    pub(super) attempt: UiMountedPresentationAttemptIdentity,
    pub(super) deadline: UiPresentationDeadline,
    pub(super) host: UiHostEffectPort<'host>,
    pub(super) authority: UiMountedHostPresentationAuthority<'authority>,
}

#[derive(Default)]
pub(super) struct UiMountedPresentationProgress {
    pub(super) pending: Vec<super::super::state::UiPendingMountedSurface>,
    pub(super) rejected: Vec<UiMountedSurfacePresentationRejection>,
    pub(super) completed: Vec<UiMountedSurfacePresentationReceipt>,
    pub(super) superseded_costs: Vec<worth_ui_host_contract::UiHostPresentationCostReport>,
    pub(super) semantic_requests: Vec<worth_ui_query_binding::WorthUiPresentationRequestBasis>,
    pub(super) superseded: bool,
}

pub(super) fn present_one_surface(
    start: &UiMountedPresentationStart<'_, '_>,
    surface: &crate::mounting::UiMountedSurfaceReceipt,
    presentation_work: &super::super::UiMountedPresentationWork,
    expected_effects: &[worth_ui_host_contract::UiMountedEffectFamily],
    progress: &mut UiMountedPresentationProgress,
    text: &mut crate::native_platform::text_presentation::UiNativeMountedTextCoordinator,
    mut presentation_async: Option<
        &mut crate::native_platform::text_presentation::UiPresentationAsyncRuntime,
    >,
) -> Result<(), UiIndeterminatePresentationEvidence> {
    let requirement = surface.requirement();
    if native_host_owns_semantic_text_boundary(
        start.host.adapter().operational_host_contract().kind(),
    ) {
        let contains_semantic_text = presentation_work.view().contains_semantic_text();
        if let Some(presentation_async_runtime) = presentation_async.as_deref_mut() {
            if let Some(observation) = super::semantic_text_raster::present(
                start,
                requirement,
                presentation_work,
                progress,
                text,
                presentation_async_runtime,
            ) {
                let (outcome, text_candidate, request_bases, pending_receipts, foreground_reuse) =
                    observation.into_parts();
                progress.semantic_requests.extend(request_bases);
                return super::presentation_outcome::record(
                    start,
                    surface,
                    expected_effects,
                    progress,
                    outcome,
                    text_candidate,
                    pending_receipts,
                    foreground_reuse,
                    text,
                    Some(presentation_async_runtime),
                );
            }
        }
        if contains_semantic_text {
            progress
                .rejected
                .push(UiMountedSurfacePresentationRejection::new(
                    requirement.binding(),
                    UiHostSurfacePresentationDenial::AdapterDeclined,
                ));
            return Ok(());
        }
        if progress
            .rejected
            .iter()
            .any(|rejection| rejection.binding() == requirement.binding())
        {
            return Ok(());
        }
    }
    let view = start.authority.bind(UiRuntimeMountedFrameConsumptionInput {
        attempt: start.attempt,
        deadline: start.deadline,
        requirement,
        presentation_work,
        text_raster_work: None,
    });
    let outcome = start
        .host
        .adapter()
        .present_mounted_surface(start.host.authority(), &view);
    super::presentation_outcome::record(
        start,
        surface,
        expected_effects,
        progress,
        outcome,
        None,
        Box::new([]),
        None,
        text,
        presentation_async,
    )
}

fn native_host_owns_semantic_text_boundary(kind: WorthUiHostKind) -> bool {
    kind == WorthUiHostKind::Native
}

pub(super) fn map_text_readiness(
    readiness: UiNativeTextPresentationReadiness,
) -> UiHostSurfacePresentationDenial {
    match readiness {
        UiNativeTextPresentationReadiness::DemandDenied(_)
        | UiNativeTextPresentationReadiness::LayoutMismatch
        | UiNativeTextPresentationReadiness::SourceMismatch => {
            UiHostSurfacePresentationDenial::AdapterDeclined
        }
    }
}

#[cfg(test)]
mod tests {
    use super::native_host_owns_semantic_text_boundary;
    use worth_ui_host_contract::WorthUiHostKind;

    #[test]
    fn semantic_text_boundary_is_native_only() {
        assert!(native_host_owns_semantic_text_boundary(
            WorthUiHostKind::Native
        ));
        assert!(!native_host_owns_semantic_text_boundary(
            WorthUiHostKind::Headless
        ));
        assert!(!native_host_owns_semantic_text_boundary(
            WorthUiHostKind::CapabilityProbeInconclusive
        ));
        assert!(!native_host_owns_semantic_text_boundary(
            WorthUiHostKind::DiagnosticsOnly
        ));
    }
}
