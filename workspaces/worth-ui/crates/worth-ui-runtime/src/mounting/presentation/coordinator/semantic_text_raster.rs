use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiHostSurfacePresentationOutcome,
    UiMountedSurfaceBindingRequirement,
};

use super::presentation_attempt::{UiMountedPresentationProgress, UiMountedPresentationStart};
use crate::native_platform::text_presentation::{
    derive_text_presentation_request_bases, UiMountedEventTimeDpiAuthority,
    UiNativeMountedTextCoordinator, UiNativeTextPresentationPreparation,
};

pub(super) fn present(
    start: &UiMountedPresentationStart<'_, '_>,
    requirement: UiMountedSurfaceBindingRequirement,
    presentation_work: &super::super::UiMountedPresentationWork,
    progress: &mut UiMountedPresentationProgress,
    text: &mut UiNativeMountedTextCoordinator,
    presentation_async: &mut crate::native_platform::text_presentation::UiPresentationAsyncRuntime,
) -> Option<crate::native_platform::text_presentation::UiNativeMountedSurfaceTextObservation> {
    let Some(dpi) = UiMountedEventTimeDpiAuthority::from_requirement(requirement) else {
        record_rejection(
            progress,
            requirement,
            UiHostSurfacePresentationDenial::AdapterDeclined,
        );
        return None;
    };
    let host_lineage = start
        .authority
        .bind(
            super::super::consumption_view::UiRuntimeMountedFrameConsumptionInput {
                attempt: start.attempt,
                deadline: start.deadline,
                requirement,
                presentation_work,
                text_raster_work: None,
            },
        )
        .host_presentation_lineage();
    let preparation = text.prepare_mounted_semantic_text(
        presentation_work.view(),
        requirement,
        dpi,
        host_lineage,
        |identity| presentation_work.resolve_layout(identity),
    );
    let preparation = preparation?;
    let prepared = match preparation {
        UiNativeTextPresentationPreparation::Prepared(prepared) => prepared,
        UiNativeTextPresentationPreparation::Denied(denial) => {
            record_rejection(
                progress,
                requirement,
                super::presentation_attempt::map_text_readiness(denial.readiness()),
            );
            return None;
        }
    };
    text.present_with_mounted_work(
        requirement.binding(),
        presentation_work.view(),
        requirement,
        host_lineage,
        &prepared,
        |identity| presentation_work.resolve_layout(identity),
        |text_raster_work| {
            let view = start.authority.bind(
                super::super::consumption_view::UiRuntimeMountedFrameConsumptionInput {
                    attempt: start.attempt,
                    deadline: start.deadline,
                    requirement,
                    presentation_work,
                    text_raster_work: Some(text_raster_work),
                },
            );
            let request_bases = match derive_text_presentation_request_bases(
                &view,
                &prepared,
                text_raster_work.pins(),
                text_raster_work.binding_pins(),
            ) {
                Ok(request_bases) => request_bases,
                Err(_) => {
                    return (
                        UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                            UiHostSurfacePresentationDenial::AdapterDeclined,
                        ),
                        Box::new([]),
                        Box::new([]),
                    );
                }
            };
            let mut pending_receipts = Vec::with_capacity(request_bases.len());
            for basis in request_bases.iter().cloned() {
                match presentation_async.admit_pending(basis) {
                    Ok(receipt) => pending_receipts.push(receipt.into()),
                    Err(denial) => {
                        if let Some(receipt) = denial.into_recovery_receipt() {
                            pending_receipts.push(receipt.into());
                        }
                        let unresolved = pending_receipts
                            .into_iter()
                            .filter(|receipt| {
                                presentation_async
                                    .reject_recovery_before_effects(receipt)
                                    .is_err()
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice();
                        let outcome = UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                            UiHostSurfacePresentationDenial::AdapterDeclined,
                        );
                        return (outcome, request_bases, unresolved);
                    }
                }
            }
            let outcome = start
                .host
                .adapter()
                .present_mounted_surface(start.host.authority(), &view);
            (outcome, request_bases, pending_receipts.into_boxed_slice())
        },
    )
}

fn record_rejection(
    progress: &mut UiMountedPresentationProgress,
    requirement: UiMountedSurfaceBindingRequirement,
    denial: UiHostSurfacePresentationDenial,
) {
    progress
        .rejected
        .push(super::super::UiMountedSurfacePresentationRejection::new(
            requirement.binding(),
            denial,
        ));
}
