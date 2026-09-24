use worth_ui_host_contract::{
    UiHostPresentationCostInput, UiHostPresentationCostReport, UiHostSurfacePresentationDenial,
    UiMountedFrameConsumptionView, UiMountedPresentationWorkView,
};

use super::{
    UiNativePresentationAccess, UiNativePresentationInput, UiNativePresentationObservation,
    UiNativeResourceClass, UiNativeResourceRegistry,
};

pub(crate) mod appearance;
mod completed_effects;
mod damage_index;
mod damage_regions;
mod delta;
#[path = "presentation/glyph_observation.rs"]
mod glyph_observation;
mod identity_overlay;
mod initial_validation;
mod pending_settlement;
mod pending_wgpu_readback;
mod pipeline;
pub(crate) mod port;
#[cfg(feature = "certification-support")]
mod qualified_external_obligation;
mod raster;
mod reconstruction;
mod retained_draw_list;
mod retained_evidence_copy;
mod retained_order;
mod retained_raster;
mod retained_regions;
mod sample;
#[cfg(feature = "certification-support")]
mod sample_certification;
mod settled_external_obligation;
mod surface;
pub(crate) mod surface_basis;
mod surface_failure;
pub(crate) mod text;
mod transaction_state;
mod unchanged;

use initial_validation::{initial_operations, validate_initial};
#[cfg(test)]
pub(crate) use pending_wgpu_readback::prove_pending_readback_handoff;
use pipeline::{draw_presentation_operations, draw_retained_to_surface, retained_transfer};
pub(crate) use pipeline::{presentation_pipelines, UiNativePresentationPipelines};
use raster::{rectangle_vertices, GlyphVertex, RasterRect, RasterVertex};
use retained_evidence_copy::copy_evidence_pixels;

#[cfg(feature = "certification-support")]
pub use appearance::{
    certify_mounted_surface_sample, UiNativeSurfaceSampleCertification,
    UiNativeSurfaceSampleCertificationDenial,
};
pub(crate) use completed_effects::{UiNativePaintOutcome, UiNativePresentationEffects};
pub(crate) use delta::{present_delta, UiNativeDeltaPresentation};
pub(crate) use pending_settlement::{
    UiNativePendingDeltaSettlement, UiNativePendingSurfaceSettlement,
    UiNativePendingUnchangedSettlement,
};
pub(crate) use pending_wgpu_readback::{UiNativePendingWgpuObligation, UiNativeWgpuReadbackPoll};
pub(crate) use port::{
    UiNativePresentationPort, UiNativePresentationPortFailure, UiNativePresentationPortObservation,
    UiNativePresentationPortPlan, UiNativeRasterOperation, UiNativeSurfaceAcquireFailure,
    UiWgpuNativePresentationPort,
};
#[cfg(feature = "certification-support")]
use qualified_external_obligation::UiNativeQualifiedExternalObligation;
pub(crate) use reconstruction::{present_cold_reconstruction, UiNativeReconstructionFailure};
#[cfg(test)]
pub(in crate::native) use retained_draw_list::tests::DrawListWorld;
pub(crate) use retained_draw_list::UiNativeRetainedDrawList;
pub(crate) use sample::{present_sample, UiNativeSamplePresentation};
#[cfg(feature = "certification-support")]
pub use sample_certification::{
    certify_portal_sample_replay, UiNativePortalSampleReplayCertification,
    UiNativePortalSampleReplayCertificationDenial,
};
pub(crate) use surface::{
    UiNativeOwnedPresentationSurface, UiNativePresentationSurface,
    UiNativePresentationSurfaceOwners,
};
pub(crate) use surface_basis::UiNativeSurfaceBasisDisposition;
#[cfg(not(feature = "certification-support"))]
pub(crate) use surface_failure::UiNativePresentationRecoveryClass;
#[cfg(feature = "certification-support")]
pub use surface_failure::{
    classify_presentation_fault, UiNativePresentationFault, UiNativePresentationFaultDisposition,
    UiNativePresentationRecoveryClass,
};
pub(crate) use surface_failure::{classify_surface_failure, UiNativeSurfaceFailureDisposition};
pub(crate) use transaction_state::UiNativePendingPresentation;
pub(crate) use transaction_state::{
    reserve_presentation_owners, settle_port_result, UiNativePendingExternalObligation,
    UiNativePendingPresentationCompletion,
};
pub(crate) use unchanged::present_unchanged_appearance;

/// How long a control may wait on the GPU before calling it a failure.
///
/// Presentation itself never waits: a frame's readback settles on a later turn
/// rather than blocking the event loop on the device.
#[cfg(test)]
pub(crate) const GPU_WAIT_DEADLINE: std::time::Duration = std::time::Duration::from_millis(5_000);

pub(crate) enum UiNativePresentationFailure {
    BeforeEffects(UiHostSurfacePresentationDenial),
    RecoveryRequired {
        denial: UiHostSurfacePresentationDenial,
        cause: super::UiNativeRecoveryCause,
    },
    Pending(UiNativePendingPresentation),
}

pub(crate) struct UiNativePresentedFrame {
    observation: super::UiNativeRetainedFrameObservation,
    retained: UiNativeRetainedDrawList,
}

impl UiNativePresentedFrame {
    pub(crate) fn into_parts(
        self,
    ) -> (
        super::UiNativeRetainedFrameObservation,
        UiNativeRetainedDrawList,
    ) {
        (self.observation, self.retained)
    }
}

pub(crate) fn present_initial<Port: UiNativePresentationPort>(
    graphics: &mut UiNativePresentationAccess,
    resources: &mut UiNativeResourceRegistry,
    physical_signal: &mut crate::native::physical_work_signal::UiNativePhysicalSignalOwner,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    atlas_gpu: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
    view: &UiMountedFrameConsumptionView<'_>,
    defer_initial_observation: bool,
    lifecycle: &mut crate::native::lifecycle::UiNativeLifecycleOrchestrator,
) -> Result<UiNativePresentedFrame, UiNativePresentationFailure> {
    let UiMountedPresentationWorkView::Initial(initial_work) = view.presentation_work() else {
        return Err(UiNativePresentationFailure::BeforeEffects(
            UiHostSurfacePresentationDenial::AdapterDeclined,
        ));
    };
    let glyph_runs = view
        .text_raster_work()
        .map(|work| work.glyph_runs())
        .unwrap_or_default();
    let mut retained =
        UiNativeRetainedDrawList::initial(initial_work, glyph_runs).map_err(|_| {
            UiNativePresentationFailure::BeforeEffects(
                UiHostSurfacePresentationDenial::MalformedProjection,
            )
        })?;
    if view.appearance_work().is_some() {
        retained
            .initialize_appearance(view, atlas, graphics.extent())
            .map_err(|_| {
                UiNativePresentationFailure::BeforeEffects(
                    UiHostSurfacePresentationDenial::MalformedProjection,
                )
            })?;
    }
    let initial =
        validate_initial(view, &retained).map_err(UiNativePresentationFailure::BeforeEffects)?;
    let mut operations = initial_operations(&retained, graphics, atlas, &initial)?;
    retained
        .initialize_physical_coverage(
            raster::UiNativeRasterBasis::from_presentation_access(graphics),
            atlas,
        )
        .map_err(|_| {
            UiNativePresentationFailure::BeforeEffects(
                UiHostSurfacePresentationDenial::MalformedProjection,
            )
        })?;
    operations.extend(
        retained
            .identity_overlay_operations(raster::UiNativeRasterBasis::from_presentation_access(
                graphics,
            ))
            .map_err(UiNativePresentationFailure::BeforeEffects)?,
    );
    let cost = initial_presentation_cost(graphics.extent(), &operations);
    let owners = reserve_presentation_owners(
        resources,
        physical_signal,
        crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::from_view(view),
    )?;
    let result = Port::present(
        graphics,
        atlas_gpu,
        UiNativePresentationPortPlan {
            clear_retained_target: true,
            operations: operations.into_boxed_slice(),
            cost,
        },
        defer_initial_observation,
        lifecycle,
    );
    let external = match settle_port_result(resources, physical_signal, owners, result) {
        Ok(external) => external,
        Err(UiNativePresentationFailure::Pending(pending)) => {
            return Err(UiNativePresentationFailure::Pending(
                pending.with_settlement(UiNativePendingSurfaceSettlement::Initial(Box::new(
                    retained,
                ))),
            ));
        }
        Err(failure) => return Err(failure),
    };
    Ok(build_presented_frame(
        view, graphics, atlas, external, retained,
    ))
}

fn build_presented_frame(
    view: &UiMountedFrameConsumptionView<'_>,
    graphics: &UiNativePresentationAccess,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    external: port::UiNativePresentationPortObservation,
    retained: UiNativeRetainedDrawList,
) -> UiNativePresentedFrame {
    let (pixels, cost, port_crossings) = external.into_parts();
    let (observation, intrinsic, alpha) = observation_for_retained(
        view,
        graphics,
        atlas,
        &retained,
        pixels,
        cost,
        port_crossings,
    );
    UiNativePresentedFrame {
        observation: super::UiNativeRetainedFrameObservation::observed(
            view.frame().diagnostic_value(),
            super::physical_work_signal::UiNativePhysicalPresentationBasis::from_view(view),
            super::UiNativePresentationWorkKind::Initial,
            None,
            pixels,
            cost,
            port_crossings,
            observation,
            intrinsic,
            alpha,
        ),
        retained,
    }
}

pub(crate) fn observation_for_retained(
    view: &UiMountedFrameConsumptionView<'_>,
    graphics: &UiNativePresentationAccess,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    retained: &UiNativeRetainedDrawList,
    pixels: [[u8; 4]; 2],
    cost: UiHostPresentationCostReport,
    port_crossings: u8,
) -> (
    Option<UiNativePresentationObservation>,
    std::sync::Arc<[super::UiNativeGlyphObservation]>,
    std::sync::Arc<[super::UiNativeGlyphObservation]>,
) {
    let glyphs = glyph_observation::observe(retained, atlas, graphics.extent());
    let (intrinsic, alpha) = (glyphs.intrinsic, glyphs.alpha);
    let observation = retained
        .top_paint_attribution()
        .map(|(ordinal, attribution)| {
            observation_for_attribution(
                view,
                graphics,
                attribution,
                ordinal,
                pixels,
                cost,
                port_crossings,
                intrinsic.clone(),
                alpha.clone(),
            )
        });
    (observation, intrinsic, alpha)
}

fn observation_for_attribution(
    view: &UiMountedFrameConsumptionView<'_>,
    graphics: &UiNativePresentationAccess,
    attribution: retained_draw_list::UiNativeRetainedPresentationAttribution,
    order_ordinal: usize,
    pixels: [[u8; 4]; 2],
    cost: UiHostPresentationCostReport,
    port_crossings: u8,
    intrinsic_glyphs: std::sync::Arc<[super::UiNativeGlyphObservation]>,
    alpha_glyphs: std::sync::Arc<[super::UiNativeGlyphObservation]>,
) -> UiNativePresentationObservation {
    let [retained_baseline_rgba8, retained_center_rgba8] = pixels;
    let bounds = attribution.bounds;
    UiNativePresentationObservation::new(UiNativePresentationInput {
        client_physical_size: graphics.extent(),
        scale_factor_milli: (graphics.scale_factor() * 1_000.0).round() as u32,
        source_rgba8: attribution.color.channels(),
        retained_center_rgba8,
        retained_baseline_rgba8,
        presented_frame: view.frame().diagnostic_value(),
        semantic_surface: view.surface().diagnostic_value(),
        host_surface: view.requirement().host_surface().diagnostic_value(),
        binding_generation: view.binding().diagnostic_value(),
        mounted_instance: attribution.mounted_instance.diagnostic_value(),
        node_receipt: attribution.node_receipt.diagnostic_value(),
        presentation_attempt: view.attempt().diagnostic_value(),
        logical_bounds_milli: [
            milli(bounds.x()),
            milli(bounds.y()),
            milli(bounds.width()),
            milli(bounds.height()),
        ],
        order_ordinal: u16::try_from(order_ordinal).expect("native profile bounds paint order"),
        port_crossings,
        production_cost: view.presentation_work().production_cost(),
        cost,
        alpha_glyphs,
        intrinsic_glyphs,
    })
}

fn milli(value: f32) -> i64 {
    (f64::from(value) * 1_000.0).round() as i64
}

fn initial_presentation_cost(
    extent: [u32; 2],
    operations: &[UiNativeRasterOperation],
) -> UiHostPresentationCostReport {
    let pixels = u64::from(extent[0]) * u64::from(extent[1]);
    let rows = u64::try_from(operations.len()).expect("native profile bounds presentation rows");
    let rendered_pixels = operations.iter().fold(0_u64, |total, operation| {
        let [width, height] = match operation {
            UiNativeRasterOperation::Clear(rect)
            | UiNativeRasterOperation::FilledRect { rect, .. } => {
                [rect.physical_width, rect.physical_height]
            }
            UiNativeRasterOperation::Surface(surface) => {
                let rect = surface.rect();
                [rect.physical_width, rect.physical_height]
            }
            UiNativeRasterOperation::Glyph(command) => [
                command.target[2].ceil().max(0.0) as u32,
                command.target[3].ceil().max(0.0) as u32,
            ],
        };
        total + u64::from(width) * u64::from(height)
    });
    UiHostPresentationCostReport::from_adapter(UiHostPresentationCostInput {
        presented_surfaces: 1,
        translated_rows: rows,
        native_resource_cache_misses: 1,
        intersecting_commands: rows,
        replayed_commands: rows,
        cleared_pixels: pixels,
        rendered_pixels,
        presented_pixels: pixels,
        gpu_writes: u64::from(!operations.is_empty()),
        render_passes: 2,
        surface_acquisitions: 1,
        queue_submissions: 1,
        presents: 1,
        ..Default::default()
    })
}

#[cfg(test)]
#[path = "presentation/pipeline_glyph_tests.rs"]
mod pipeline_glyph_tests;
#[cfg(test)]
#[path = "presentation_tests.rs"]
mod tests;

#[cfg(feature = "certification-support")]
pub use retained_draw_list::UiNativeTextReplayOperation;
