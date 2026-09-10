use worth_ui_host_contract::{
    UiHostPresentationCostReport, UiHostSurfacePresentationDenial, UiMountedFrameConsumptionView,
    UiMountedPresentationWorkView,
};

use super::port::UiNativePresentationPortObservation;
use super::raster::UiNativeRasterBasis;
use super::retained_draw_list::UiNativeRetainedDeltaUndo;
use super::retained_raster::build_plan;
use super::{
    reserve_presentation_owners, settle_port_result, UiNativePresentationFailure,
    UiNativePresentationPort, UiNativePresentationPortPlan, UiNativeRetainedDrawList,
};
use crate::native::{UiNativePresentationAccess, UiNativeResourceRegistry};

pub(crate) struct UiNativeDeltaPresentation {
    cost: UiHostPresentationCostReport,
    painted: bool,
    pixels: Option<[[u8; 4]; 2]>,
    port_crossings: u8,
    effects: super::UiNativePresentationEffects,
}

impl UiNativeDeltaPresentation {
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

pub(crate) fn present_delta<Port: UiNativePresentationPort>(
    graphics: &mut UiNativePresentationAccess,
    resources: &mut UiNativeResourceRegistry,
    physical_signal: &mut crate::native::physical_work_signal::UiNativePhysicalSignalOwner,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    atlas_gpu: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
    view: &UiMountedFrameConsumptionView<'_>,
    retained: &mut UiNativeRetainedDrawList,
    defer_initial_observation: bool,
    lifecycle: &mut crate::native::lifecycle::UiNativeLifecycleOrchestrator,
) -> Result<UiNativeDeltaPresentation, UiNativePresentationFailure> {
    let UiMountedPresentationWorkView::Delta(delta) = view.presentation_work() else {
        return Err(before_effects(
            UiHostSurfacePresentationDenial::AdapterDeclined,
        ));
    };
    let basis = UiNativeRasterBasis::from_presentation_access(graphics);
    let glyph_runs = view
        .text_raster_work()
        .map(|work| work.glyph_runs())
        .unwrap_or_default();
    let (plan, undo, effects) = if view.appearance_work().is_some() {
        prepare_delta_plan_with_appearance(basis, delta, glyph_runs, atlas, view, retained)?
    } else {
        prepare_delta_plan(basis, delta, glyph_runs, atlas, retained)?
    };
    if plan.operations.is_empty() && !plan.clear_retained_target {
        return Ok(UiNativeDeltaPresentation {
            cost: plan.cost,
            painted: false,
            pixels: None,
            port_crossings: 0,
            effects: effects.without_native_paint(),
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
                .rollback_delta(undo)
                .expect("capacity refusal must preserve retained state");
            return Err(failure);
        }
    };
    settle_staged_delta(
        retained,
        undo,
        effects,
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

fn prepare_delta_plan_with_appearance(
    basis: UiNativeRasterBasis,
    delta: &worth_ui_host_contract::UiMountedPresentationDelta,
    glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    view: &UiMountedFrameConsumptionView<'_>,
    retained: &mut UiNativeRetainedDrawList,
) -> Result<
    (
        UiNativePresentationPortPlan,
        UiNativeRetainedDeltaUndo,
        super::UiNativePresentationEffects,
    ),
    UiNativePresentationFailure,
> {
    let work = view
        .appearance_work()
        .ok_or_else(|| before_effects(UiHostSurfacePresentationDenial::MalformedProjection))?;
    let (mut replay, mut undo) = retained
        .stage_delta(delta, glyph_runs)
        .map_err(|_| before_effects(UiHostSurfacePresentationDenial::MalformedProjection))?;
    let effects = super::UiNativePresentationEffects::new(
        !delta.changes().is_empty() || !delta.order().is_empty() || !delta.damage().is_empty(),
        replay.identity_overlay_effect,
    );
    let staged = (|| {
        retained.refresh_physical_delta(delta, &mut undo, basis, atlas, &mut replay)?;
        for fragment in work.fragments() {
            let candidates =
                changed_text_foregrounds(fragment, view, atlas, basis.extent(), retained)?;
            if fragment.work().changes().iter().any(is_text_change) {
                retained.stage_text_coverage_fragment_after_delta(
                    delta,
                    fragment,
                    view.attempt(),
                    candidates,
                    atlas,
                    &mut undo,
                )?;
            }
            for staged in retained.stage_nontext_appearance_fragment(fragment)? {
                undo.retain_appearance_command(staged);
            }
        }
        let overlay = work
            .fragments()
            .first()
            .ok_or(super::retained_draw_list::UiNativeRetainedDrawListDenial::CommandMismatch)?
            .work()
            .successor()
            .overlay_order();
        undo.retain_appearance_command(retained.stage_appearance_overlay(overlay)?);
        let samples = retained.stage_appearance_sample_overrides(work.sample_overrides())?;
        undo.retain_appearance_samples(samples);
        replay.staged_appearance_regions = retained.prepare_appearance_replay()?;
        build_plan(basis, retained, replay, delta.nodes().len(), atlas)
            .map_err(|_| super::retained_draw_list::UiNativeRetainedDrawListDenial::CommandMismatch)
    })();
    match staged {
        Ok(plan) => Ok((plan, undo, effects)),
        Err(_) => {
            retained
                .rollback_delta(undo)
                .expect("appearance delta refusal must preserve retained state");
            Err(before_effects(
                UiHostSurfacePresentationDenial::MalformedProjection,
            ))
        }
    }
}

pub(super) fn is_text_change(
    change: &worth_ui_host_contract::UiMountedAppearanceMechanicChange,
) -> bool {
    use worth_ui_host_contract::{
        UiMountedAppearanceMechanic as Mechanic, UiMountedAppearanceMechanicChange as Change,
        UiMountedAppearanceMechanicIdentity as Identity,
    };
    matches!(
        change,
        Change::Insert(Mechanic::TextForeground(_))
            | Change::Replace {
                successor: Mechanic::TextForeground(_),
                ..
            }
            | Change::Remove(Identity::TextForeground { .. })
    )
}

pub(super) fn changed_text_foregrounds(
    fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
    view: &UiMountedFrameConsumptionView<'_>,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    extent: [u32; 2],
    retained: &UiNativeRetainedDrawList,
) -> Result<
    Vec<crate::native::presentation::appearance::text_foreground::UiNativeFinalizedTextForeground>,
    super::retained_draw_list::UiNativeRetainedDrawListDenial,
> {
    use worth_ui_host_contract::{
        UiMountedAppearanceMechanic as Mechanic, UiMountedAppearanceMechanicChange as Change,
    };
    fragment
        .work()
        .changes()
        .iter()
        .filter_map(|change| match change {
            Change::Insert(Mechanic::TextForeground(value))
            | Change::Replace {
                successor: Mechanic::TextForeground(value),
                ..
            } => Some(value),
            _ => None,
        })
        .map(|mechanic| {
            let finalized = crate::native::presentation::appearance::text_foreground::UiNativeTextForegroundJoin::admit(
                fragment, view, mechanic,
            )
            .and_then(|join| join.finalize(atlas, extent))
            .map(|(foreground, _)| foreground);
            match finalized {
                Ok(foreground) => Ok(foreground),
                Err(
                    crate::native::presentation::appearance::text_foreground::UiNativeTextForegroundFinalizationDenial::MissingRasterWork
                    | crate::native::presentation::appearance::text_foreground::UiNativeTextForegroundFinalizationDenial::UnauthenticatedDemand,
                ) => retained
                    .inherit_text_foreground_replacement(fragment, view, mechanic, atlas)
                    .map_err(|denial| denial),
                Err(_) => {
                    Err(super::retained_draw_list::UiNativeRetainedDrawListDenial::CommandMismatch)
                }
            }
        })
        .collect()
}

pub(super) fn prepare_delta_plan(
    basis: UiNativeRasterBasis,
    delta: &worth_ui_host_contract::UiMountedPresentationDelta,
    glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    retained: &mut UiNativeRetainedDrawList,
) -> Result<
    (
        UiNativePresentationPortPlan,
        UiNativeRetainedDeltaUndo,
        super::UiNativePresentationEffects,
    ),
    UiNativePresentationFailure,
> {
    let (mut replay, mut undo) = retained
        .stage_delta(delta, glyph_runs)
        .map_err(|_| before_effects(UiHostSurfacePresentationDenial::MalformedProjection))?;
    let effects = super::UiNativePresentationEffects::new(
        !delta.changes().is_empty() || !delta.order().is_empty() || !delta.damage().is_empty(),
        replay.identity_overlay_effect,
    );
    let prepared = retained
        .refresh_physical_delta(delta, &mut undo, basis, atlas, &mut replay)
        .map_err(|_| UiHostSurfacePresentationDenial::MalformedProjection)
        .and_then(|()| build_plan(basis, retained, replay, delta.nodes().len(), atlas));
    match prepared.map_err(before_effects) {
        Ok(plan) => Ok((plan, undo, effects)),
        Err(failure) => {
            retained
                .rollback_delta(undo)
                .expect("a prevalidated native delta must roll back exactly");
            Err(failure)
        }
    }
}

pub(crate) fn settle_staged_delta(
    retained: &mut UiNativeRetainedDrawList,
    undo: UiNativeRetainedDeltaUndo,
    effects: super::UiNativePresentationEffects,
    result: Result<UiNativePresentationPortObservation, UiNativePresentationFailure>,
) -> Result<UiNativeDeltaPresentation, UiNativePresentationFailure> {
    match result {
        Ok(observation) => {
            let (pixels, cost, port_crossings) = observation.into_parts();
            Ok(UiNativeDeltaPresentation {
                cost,
                painted: true,
                pixels: Some(pixels),
                port_crossings,
                effects,
            })
        }
        Err(
            failure @ (UiNativePresentationFailure::BeforeEffects(_)
            | UiNativePresentationFailure::RecoveryRequired { .. }),
        ) => {
            retained
                .rollback_delta(undo)
                .expect("before-effect port refusal must preserve retained state");
            Err(failure)
        }
        Err(UiNativePresentationFailure::Pending(pending)) => {
            Err(UiNativePresentationFailure::Pending(
                pending.with_settlement(super::UiNativePendingSurfaceSettlement::Delta(
                    super::UiNativePendingDeltaSettlement::new(undo, effects),
                )),
            ))
        }
    }
}

fn before_effects(denial: UiHostSurfacePresentationDenial) -> UiNativePresentationFailure {
    UiNativePresentationFailure::BeforeEffects(denial)
}

#[cfg(test)]
#[path = "delta_tests.rs"]
mod tests;
