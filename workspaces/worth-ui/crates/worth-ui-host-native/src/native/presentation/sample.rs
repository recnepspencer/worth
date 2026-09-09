use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedCanonicalBox, UiMountedFrameConsumptionView,
    UiMountedPaintCommand, UiMountedPresentationSampleChange, UiMountedPresentationTransform,
    UiMountedPresentationWorkView,
};

use super::port::UiNativePresentationPortObservation;
use super::raster::UiNativeRasterBasis;
use super::retained_draw_list::UiNativeRetainedSampleUndo;
use super::retained_raster::build_plan;
use super::{
    reserve_presentation_owners, settle_port_result, UiNativePresentationFailure,
    UiNativePresentationPort, UiNativeRetainedDrawList,
};
use crate::native::{UiNativePresentationAccess, UiNativeResourceRegistry};

pub(crate) struct UiNativeSamplePresentation {
    cost: worth_ui_host_contract::UiHostPresentationCostReport,
    painted: bool,
    pixels: Option<[[u8; 4]; 2]>,
    port_crossings: u8,
    effects: super::UiNativePresentationEffects,
}

impl UiNativeSamplePresentation {
    pub(crate) const fn into_parts(
        self,
    ) -> (
        worth_ui_host_contract::UiHostPresentationCostReport,
        bool,
        Option<[[u8; 4]; 2]>,
        u8,
        super::UiNativePresentationEffects,
    ) {
        (
            self.cost,
            self.painted,
            self.pixels,
            self.port_crossings,
            self.effects,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn present_sample<Port: UiNativePresentationPort>(
    graphics: &mut UiNativePresentationAccess,
    resources: &mut UiNativeResourceRegistry,
    physical_signal: &mut crate::native::physical_work_signal::UiNativePhysicalSignalOwner,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    atlas_gpu: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
    view: &UiMountedFrameConsumptionView<'_>,
    retained: &mut UiNativeRetainedDrawList,
    defer_initial_observation: bool,
    lifecycle: &mut crate::native::lifecycle::UiNativeLifecycleOrchestrator,
) -> Result<UiNativeSamplePresentation, UiNativePresentationFailure> {
    let UiMountedPresentationWorkView::Sample(sample) = view.presentation_work() else {
        return Err(before_effects(
            UiHostSurfacePresentationDenial::AdapterDeclined,
        ));
    };
    let basis = UiNativeRasterBasis::from_presentation_access(graphics);
    let (plan, undo) = prepare_sample_plan(basis, sample, atlas, retained)?;
    if plan.operations.is_empty() && !plan.clear_retained_target {
        return Ok(UiNativeSamplePresentation {
            cost: plan.cost,
            painted: false,
            pixels: None,
            port_crossings: 0,
            effects: super::UiNativePresentationEffects::new(true, false).without_native_paint(),
        });
    }
    let owners = match reserve_presentation_owners(
        resources,
        physical_signal,
        crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::from_view(view),
    ) {
        Ok(owners) => owners,
        Err(failure) => {
            retained
                .rollback_sample(undo)
                .expect("capacity refusal preserves the previous sample");
            return Err(failure);
        }
    };
    settle_staged_sample(
        retained,
        undo,
        settle_port_result(
            resources,
            physical_signal,
            owners,
            Port::present(
                graphics,
                atlas_gpu,
                plan,
                defer_initial_observation,
                lifecycle,
            ),
        ),
    )
}

pub(super) fn prepare_sample_plan(
    basis: UiNativeRasterBasis,
    sample: &worth_ui_host_contract::UiMountedPresentationSample,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    retained: &mut UiNativeRetainedDrawList,
) -> Result<
    (
        super::UiNativePresentationPortPlan,
        UiNativeRetainedSampleUndo,
    ),
    UiNativePresentationFailure,
> {
    let (mut replay, mut undo) = retained
        .stage_sample(sample)
        .map_err(|_| before_effects(malformed()))?;
    let prepared = retained
        .refresh_physical_sample(sample, &mut undo, basis, &mut replay)
        .map_err(|_| malformed())
        .and_then(|()| build_plan(basis, retained, replay, 0, atlas));
    let plan = match prepared {
        Ok(plan) => plan,
        Err(denial) => {
            retained
                .rollback_sample(undo)
                .expect("a prevalidated native sample rolls back exactly");
            return Err(before_effects(denial));
        }
    };
    Ok((plan, undo))
}

fn settle_staged_sample(
    retained: &mut UiNativeRetainedDrawList,
    undo: UiNativeRetainedSampleUndo,
    result: Result<UiNativePresentationPortObservation, UiNativePresentationFailure>,
) -> Result<UiNativeSamplePresentation, UiNativePresentationFailure> {
    match result {
        Ok(observation) => {
            let (pixels, cost, port_crossings) = observation.into_parts();
            Ok(UiNativeSamplePresentation {
                cost,
                painted: true,
                pixels: Some(pixels),
                port_crossings,
                effects: super::UiNativePresentationEffects::new(true, false),
            })
        }
        Err(
            failure @ (UiNativePresentationFailure::BeforeEffects(_)
            | UiNativePresentationFailure::RecoveryRequired { .. }),
        ) => {
            retained
                .rollback_sample(undo)
                .expect("before-effect refusal preserves the previous sample");
            Err(failure)
        }
        Err(UiNativePresentationFailure::Pending(pending)) => {
            Err(UiNativePresentationFailure::Pending(
                pending.with_settlement(super::UiNativePendingSurfaceSettlement::Sample(undo)),
            ))
        }
    }
}

pub(super) fn sampled_command_bounds(
    command: &UiMountedPaintCommand,
    change: Option<UiMountedPresentationSampleChange>,
) -> Result<UiMountedCanonicalBox, UiHostSurfacePresentationDenial> {
    super::retained_draw_list::sampled_visible_bounds(command, change)
        .map_err(|_| malformed())?
        .ok_or_else(malformed)
}

pub(super) fn transform_physical_box(
    bounds: [f32; 4],
    transform: UiMountedPresentationTransform,
    basis: UiNativeRasterBasis,
) -> Result<[f32; 4], UiHostSurfacePresentationDenial> {
    let scale = basis.scale_factor();
    let source = transform.source();
    let sampled = transform.sampled();
    let source = [
        source.x() * scale,
        source.y() * scale,
        source.width() * scale,
        source.height() * scale,
    ];
    let sampled = [
        sampled.x() * scale,
        sampled.y() * scale,
        sampled.width() * scale,
        sampled.height() * scale,
    ];
    if source[2] <= 0.0 || source[3] <= 0.0 {
        return Err(malformed());
    }
    Ok([
        sampled[0] + (bounds[0] - source[0]) * sampled[2] / source[2],
        sampled[1] + (bounds[1] - source[1]) * sampled[3] / source[3],
        bounds[2] * sampled[2] / source[2],
        bounds[3] * sampled[3] / source[3],
    ])
}

fn malformed() -> UiHostSurfacePresentationDenial {
    UiHostSurfacePresentationDenial::MalformedProjection
}

fn before_effects(denial: UiHostSurfacePresentationDenial) -> UiNativePresentationFailure {
    UiNativePresentationFailure::BeforeEffects(denial)
}

#[cfg(test)]
#[path = "sample_tests.rs"]
mod tests;
