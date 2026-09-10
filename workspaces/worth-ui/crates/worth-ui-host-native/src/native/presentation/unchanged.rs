use worth_ui_host_contract::{
    UiHostPresentationCostReport, UiHostSurfacePresentationDenial, UiMountedFrameConsumptionView,
    UiMountedPresentationUnchanged,
};

use super::{
    reserve_presentation_owners, retained_raster::build_plan, settle_port_result,
    UiNativePresentationFailure, UiNativePresentationPort, UiNativeRetainedDrawList,
};
use crate::native::{UiNativePresentationAccess, UiNativeResourceRegistry};

pub(crate) struct UiNativeUnchangedPresentation {
    cost: UiHostPresentationCostReport,
    painted: bool,
    pixels: Option<[[u8; 4]; 2]>,
    port_crossings: u8,
    effects: super::UiNativePresentationEffects,
}

impl UiNativeUnchangedPresentation {
    pub(crate) fn into_parts(
        self,
    ) -> (
        UiHostPresentationCostReport,
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
pub(crate) fn present_unchanged_appearance<Port: UiNativePresentationPort>(
    graphics: &mut UiNativePresentationAccess,
    resources: &mut UiNativeResourceRegistry,
    physical_signal: &mut crate::native::physical_work_signal::UiNativePhysicalSignalOwner,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    atlas_gpu: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
    view: &UiMountedFrameConsumptionView<'_>,
    unchanged: &UiMountedPresentationUnchanged,
    retained: &mut UiNativeRetainedDrawList,
    defer_initial_observation: bool,
    lifecycle: &mut crate::native::lifecycle::UiNativeLifecycleOrchestrator,
) -> Result<UiNativeUnchangedPresentation, UiNativePresentationFailure> {
    let basis = super::raster::UiNativeRasterBasis::from_presentation_access(graphics);
    let mut undo = retained
        .stage_unchanged(unchanged)
        .map_err(|_| malformed())?;
    let staged = stage_appearance(view, unchanged, atlas, basis.extent(), retained, &mut undo)
        .and_then(|()| {
            retained
                .appearance_only_replay_plan()
                .map_err(|_| malformed())
        })
        .and_then(|replay| {
            build_plan(basis, retained, replay, 0, atlas)
                .map_err(super::UiNativePresentationFailure::BeforeEffects)
        });
    let plan = match staged {
        Ok(plan) => plan,
        Err(failure) => {
            if let Err(denial) = retained.rollback_unchanged(undo) {
                panic!("appearance-only refusal rollback failed: {denial:?}");
            }
            return Err(failure);
        }
    };
    let effects = super::UiNativePresentationEffects::new(!plan.operations.is_empty(), false);
    if plan.operations.is_empty() && !plan.clear_retained_target {
        return Ok(UiNativeUnchangedPresentation {
            cost: plan.cost,
            painted: false,
            pixels: None,
            port_crossings: 0,
            effects,
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
                .rollback_unchanged(undo)
                .expect("capacity refusal must restore appearance-only predecessor truth");
            return Err(failure);
        }
    };
    match settle_port_result(
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
    ) {
        Ok(observation) => {
            let (pixels, cost, port_crossings) = observation.into_parts();
            Ok(UiNativeUnchangedPresentation {
                cost,
                painted: true,
                pixels: Some(pixels),
                port_crossings,
                effects,
            })
        }
        Err(UiNativePresentationFailure::Pending(pending)) => {
            Err(UiNativePresentationFailure::Pending(
                pending.with_settlement(super::UiNativePendingSurfaceSettlement::Unchanged(
                    super::UiNativePendingUnchangedSettlement::new(undo, effects),
                )),
            ))
        }
        Err(failure) => {
            retained
                .rollback_unchanged(undo)
                .expect("before-effect refusal must restore appearance-only predecessor truth");
            Err(failure)
        }
    }
}

fn stage_appearance(
    view: &UiMountedFrameConsumptionView<'_>,
    unchanged: &UiMountedPresentationUnchanged,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    extent: [u32; 2],
    retained: &mut UiNativeRetainedDrawList,
    undo: &mut super::retained_draw_list::UiNativeRetainedUnchangedUndo,
) -> Result<(), UiNativePresentationFailure> {
    let work = view.appearance_work().ok_or_else(malformed)?;
    if work.frame() != view.frame() || work.requirement() != view.requirement() {
        return Err(malformed());
    }
    for fragment in work.fragments() {
        let candidates =
            super::delta::changed_text_foregrounds(fragment, view, atlas, extent, retained)
                .map_err(|_| malformed())?;
        if fragment
            .work()
            .changes()
            .iter()
            .any(super::delta::is_text_change)
        {
            retained
                .stage_text_coverage_fragment_after_unchanged(
                    unchanged,
                    fragment,
                    view.attempt(),
                    candidates,
                    atlas,
                    undo,
                )
                .map_err(|_| malformed())?;
        }
        for command in retained
            .stage_nontext_appearance_fragment(fragment)
            .map_err(|_| malformed())?
        {
            UiNativeRetainedDrawList::retain_unchanged_appearance(undo, command);
        }
    }
    let overlay = work
        .fragments()
        .first()
        .ok_or_else(malformed)?
        .work()
        .successor()
        .overlay_order();
    let command = retained
        .stage_appearance_overlay(overlay)
        .map_err(|_| malformed())?;
    UiNativeRetainedDrawList::retain_unchanged_appearance(undo, command);
    let samples = retained
        .stage_appearance_sample_overrides(work.sample_overrides())
        .map_err(|_| malformed())?;
    UiNativeRetainedDrawList::retain_unchanged_appearance_samples(undo, samples);
    Ok(())
}

fn malformed() -> UiNativePresentationFailure {
    UiNativePresentationFailure::BeforeEffects(UiHostSurfacePresentationDenial::MalformedProjection)
}
