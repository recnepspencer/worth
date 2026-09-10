//! Ordered physical replay shared by delta and Motion sample presentation.
use worth_ui_host_contract::{UiHostSurfacePresentationDenial, UiMountedPaintCommand};

use super::raster::{raster_damage_for_basis, UiNativeRasterBasis};
use super::{UiNativePresentationPortPlan, UiNativeRasterOperation, UiNativeRetainedDrawList};

mod cost;
pub(super) use cost::replay_cost;

pub(super) fn build_plan(
    basis: UiNativeRasterBasis,
    retained: &mut UiNativeRetainedDrawList,
    mut replay: super::retained_draw_list::UiNativeRetainedReplayPlan,
    node_changes: usize,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
) -> Result<UiNativePresentationPortPlan, UiHostSurfacePresentationDenial> {
    validate_replay_baseline(replay.baseline_rgba8)?;
    let staged_clears = retained
        .staged_appearance_clears(&replay.staged_appearance_regions, basis)
        .map_err(|_| malformed())?;
    let logical_clears = replay.regions.iter().map(|region| {
        raster_damage_for_basis(region.damage.bounds(), basis).map_err(|_| malformed())
    });
    let mut operations = Vec::new();
    let mut cleared_pixels = 0_u64;
    let mut rendered_pixels = 0_u64;
    let mut replayed_commands = 0_u64;
    let physical_clears = replay
        .physical_text_regions
        .into_iter()
        .chain(staged_clears);
    for clear in logical_clears.chain(physical_clears.map(|clear| Ok(Some(clear)))) {
        let Some(clear) = clear? else {
            continue;
        };
        cleared_pixels = add_pixels(cleared_pixels, clear)?;
        operations.push(UiNativeRasterOperation::Clear(clear));
        let physical_replay = retained
            .physical_replay_for_damage(basis, clear.physical_bounds(), &mut replay.counters)
            .map_err(|_| malformed())?;
        if retained.has_appearance() {
            for operation in retained
                .appearance_operations_for_damage(
                    clear,
                    basis,
                    &physical_replay,
                    atlas,
                    &mut replay.counters,
                )
                .map_err(|_| malformed())?
            {
                rendered_pixels = rendered_pixels
                    .checked_add(operation_pixels(&operation))
                    .ok_or_else(malformed)?;
                operations.push(operation);
                replayed_commands = replayed_commands.checked_add(1).ok_or_else(malformed)?;
            }
            continue;
        }
        for identity in &physical_replay {
            let command = retained.command(*identity).ok_or_else(malformed)?;
            let sample = retained.sample_override(*identity);
            let opacity = sample.map_or(1.0, |sample| sample.opacity().factor());
            match command {
                UiMountedPaintCommand::FilledRect { mechanic, .. } => {
                    if let Some(UiNativeRasterOperation::Surface(surface)) = retained
                        .appearance_surface_operation(mechanic.node_receipt(), basis.extent())
                        .map_err(|_| malformed())?
                    {
                        if let Some(surface) = surface.clipped_to(clear, basis.extent()) {
                            rendered_pixels = add_pixels(rendered_pixels, surface.rect())?;
                            operations.push(UiNativeRasterOperation::Surface(surface));
                        }
                        replayed_commands =
                            replayed_commands.checked_add(1).ok_or_else(malformed)?;
                        continue;
                    }
                    let sampled = super::sample::sampled_command_bounds(command, sample)?;
                    let Some(rect) = raster_damage_for_basis(sampled, basis)
                        .map_err(|_| malformed())?
                        .and_then(|rect| rect.intersection(clear, basis.extent()))
                    else {
                        continue;
                    };
                    rendered_pixels = add_pixels(rendered_pixels, rect)?;
                    operations.push(UiNativeRasterOperation::FilledRect {
                        rect,
                        source_rgba8: sampled_color(mechanic.color().channels(), opacity),
                    });
                }
                UiMountedPaintCommand::PortalOverlay { mechanic, .. } => {
                    if let Some(UiNativeRasterOperation::Surface(surface)) = retained
                        .appearance_portal_surface_operation(mechanic.owner(), basis.extent())
                        .map_err(|_| malformed())?
                    {
                        if let Some(surface) = surface.clipped_to(clear, basis.extent()) {
                            rendered_pixels = add_pixels(rendered_pixels, surface.rect())?;
                            operations.push(UiNativeRasterOperation::Surface(surface));
                        }
                        replayed_commands =
                            replayed_commands.checked_add(1).ok_or_else(malformed)?;
                        continue;
                    }
                    let sampled = super::sample::sampled_command_bounds(command, sample)?;
                    let Some(rect) = raster_damage_for_basis(sampled, basis)
                        .map_err(|_| malformed())?
                        .and_then(|rect| rect.intersection(clear, basis.extent()))
                    else {
                        continue;
                    };
                    rendered_pixels = add_pixels(rendered_pixels, rect)?;
                    operations.push(UiNativeRasterOperation::FilledRect {
                        rect,
                        source_rgba8: sampled_color(mechanic.color().channels(), opacity),
                    });
                }
                UiMountedPaintCommand::SemanticText { .. } => {
                    let glyphs = retained
                        .plan_text_commands(*identity, atlas, basis)
                        .map_err(|_| malformed())?;
                    for glyph in glyphs.iter().copied() {
                        let Some(glyph) =
                            super::text::clip_glyph_command(glyph, clear.physical_bounds())
                        else {
                            continue;
                        };
                        rendered_pixels = rendered_pixels
                            .checked_add(
                                (glyph.target[2].ceil() as u64) * (glyph.target[3].ceil() as u64),
                            )
                            .ok_or_else(malformed)?;
                        operations.push(UiNativeRasterOperation::Glyph(glyph));
                    }
                }
            }
            replayed_commands = replayed_commands.checked_add(1).ok_or_else(malformed)?;
        }
    }
    operations.extend(retained.identity_overlay_operations(basis)?);
    let cost = replay_cost(
        basis.extent(),
        replay.counters,
        operations.len(),
        cleared_pixels,
        rendered_pixels,
        replayed_commands,
        node_changes,
    )?;
    Ok(UiNativePresentationPortPlan {
        clear_retained_target: false,
        operations: operations.into_boxed_slice(),
        cost,
    })
}

fn operation_pixels(operation: &UiNativeRasterOperation) -> u64 {
    match operation {
        UiNativeRasterOperation::Clear(_) => 0,
        UiNativeRasterOperation::FilledRect { rect, .. } => {
            u64::from(rect.physical_width) * u64::from(rect.physical_height)
        }
        UiNativeRasterOperation::Surface(surface) => {
            u64::from(surface.rect().physical_width) * u64::from(surface.rect().physical_height)
        }
        UiNativeRasterOperation::Glyph(glyph) => {
            (glyph.target[2].ceil() as u64) * (glyph.target[3].ceil() as u64)
        }
    }
}

fn validate_replay_baseline(
    baseline_rgba8: [u8; 4],
) -> Result<(), UiHostSurfacePresentationDenial> {
    (baseline_rgba8 == [0, 0, 0, 0])
        .then_some(())
        .ok_or(UiHostSurfacePresentationDenial::MalformedProjection)
}

pub(super) fn sampled_color(mut color: [u8; 4], opacity: f32) -> [u8; 4] {
    color[3] = (f32::from(color[3]) * opacity).round() as u8;
    color
}

fn add_pixels(total: u64, rect: super::RasterRect) -> Result<u64, UiHostSurfacePresentationDenial> {
    total
        .checked_add(u64::from(rect.physical_width) * u64::from(rect.physical_height))
        .ok_or_else(malformed)
}

fn malformed() -> UiHostSurfacePresentationDenial {
    UiHostSurfacePresentationDenial::MalformedProjection
}
