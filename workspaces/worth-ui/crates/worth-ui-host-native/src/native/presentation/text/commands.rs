//! Exact native glyph commands derived from runtime-issued glyph-run views.
//!
//! The command keeps draw geometry, atlas placement, and semantic attribution
//! together. Native code never reconstructs placement from a raster key or an
//! original byte range.

use worth_ui_host_contract::{UiGlyphRasterExtent, UiGlyphRasterSource, UiGlyphRunView};

use crate::native::text_atlas::{
    UiNativeGpuAtlasKind, UiNativeTextAtlas, UiNativeTextAtlasEntryView,
};

#[cfg(test)]
#[path = "command_geometry_tests.rs"]
mod command_geometry_tests;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiNativeGlyphCommand {
    pub(crate) run: UiGlyphRunView,
    pub(crate) foreground: worth_ui_host_contract::UiMountedRgba8,
    pub(crate) atlas_kind: UiNativeGpuAtlasKind,
    pub(crate) atlas_page: u32,
    pub(crate) target: [f32; 4],
    pub(crate) texture_uv: [f32; 4],
    pub(crate) opacity: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeGlyphCommandDenial {
    MissingAtlasEntry,
    GeometryOverflow,
}

pub(crate) fn plan_glyph_commands(
    runs: &[UiGlyphRunView],
    atlas: &UiNativeTextAtlas,
    target_extent: [u32; 2],
) -> Result<Box<[UiNativeGlyphCommand]>, UiNativeGlyphCommandDenial> {
    let mut commands = runs
        .iter()
        .copied()
        .enumerate()
        .map(|(ordinal, run)| {
            let entry = atlas
                .entry_view(run.raster_key())
                .ok_or(UiNativeGlyphCommandDenial::MissingAtlasEntry);
            entry.and_then(|entry| {
                command_for_run(run, entry, target_extent).map(|value| {
                    value.map(|command| (run.layer_semantic_order(), ordinal, command))
                })
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    commands.sort_by_key(|command| {
        command
            .as_ref()
            .map(|(layer, ordinal, _)| (*layer, *ordinal))
    });
    Ok(commands
        .into_iter()
        .flatten()
        .map(|(_, _, command)| command)
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn command_for_run(
    run: UiGlyphRunView,
    entry: UiNativeTextAtlasEntryView,
    target_extent: [u32; 2],
) -> Result<Option<UiNativeGlyphCommand>, UiNativeGlyphCommandDenial> {
    let extent = UiGlyphRasterExtent::new(entry.extent[0], entry.extent[1])
        .ok_or(UiNativeGlyphCommandDenial::GeometryOverflow)?;
    let raw = entry
        .bearing
        .positioned_bounds(
            extent,
            [run.origin_x_millipoints(), run.origin_y_millipoints()],
            run.raster_key().dpi_milli(),
        )
        .ok_or(UiNativeGlyphCommandDenial::GeometryOverflow)?;
    let clip = physical_clip(run, target_extent)?;
    let Some(target) = intersect(raw, clip) else {
        return Ok(None);
    };
    let relative = [
        (target[0] - raw[0]) / raw[2],
        (target[1] - raw[1]) / raw[3],
        target[2] / raw[2],
        target[3] / raw[3],
    ];
    let page_extent = [entry.page_extent[0] as f32, entry.page_extent[1] as f32];
    let texture_uv = [
        (entry.origin[0] as f32 + relative[0] * entry.extent[0] as f32) / page_extent[0],
        (entry.origin[1] as f32 + relative[1] * entry.extent[1] as f32) / page_extent[1],
        relative[2] * entry.extent[0] as f32 / page_extent[0],
        relative[3] * entry.extent[1] as f32 / page_extent[1],
    ];
    Ok(Some(UiNativeGlyphCommand {
        run,
        foreground: run.foreground(),
        atlas_kind: entry.kind,
        atlas_page: entry.page,
        target,
        texture_uv,
        opacity: 1.0,
    }))
}

fn physical_clip(
    run: UiGlyphRunView,
    target_extent: [u32; 2],
) -> Result<[f32; 4], UiNativeGlyphCommandDenial> {
    let clip = run.clip_bounds();
    let scale = run.raster_key().dpi_milli() as f64 / 1_000.0;
    let values = [
        f64::from(clip.x()) * scale,
        f64::from(clip.y()) * scale,
        f64::from(clip.width()) * scale,
        f64::from(clip.height()) * scale,
    ];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(UiNativeGlyphCommandDenial::GeometryOverflow);
    }
    let surface = [0.0, 0.0, target_extent[0] as f32, target_extent[1] as f32];
    Ok(intersect(
        [
            values[0] as f32,
            values[1] as f32,
            values[2] as f32,
            values[3] as f32,
        ],
        surface,
    )
    .unwrap_or([0.0; 4]))
}

fn intersect(left: [f32; 4], right: [f32; 4]) -> Option<[f32; 4]> {
    let x = left[0].max(right[0]);
    let y = left[1].max(right[1]);
    let far_x = (left[0] + left[2]).min(right[0] + right[2]);
    let far_y = (left[1] + left[3]).min(right[1] + right[3]);
    (far_x > x && far_y > y).then_some([x, y, far_x - x, far_y - y])
}

pub(crate) fn source_is_intrinsic_color(command: UiNativeGlyphCommand) -> bool {
    matches!(
        command.run.raster_key().source(),
        UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap
    )
}

pub(crate) fn clip_glyph_command(
    command: UiNativeGlyphCommand,
    clip: [f32; 4],
) -> Option<UiNativeGlyphCommand> {
    let target = intersect(command.target, clip)?;
    let relative = [
        (target[0] - command.target[0]) / command.target[2],
        (target[1] - command.target[1]) / command.target[3],
        target[2] / command.target[2],
        target[3] / command.target[3],
    ];
    Some(UiNativeGlyphCommand {
        target,
        texture_uv: [
            command.texture_uv[0] + relative[0] * command.texture_uv[2],
            command.texture_uv[1] + relative[1] * command.texture_uv[3],
            relative[2] * command.texture_uv[2],
            relative[3] * command.texture_uv[3],
        ],
        ..command
    })
}

pub(crate) fn glyph_vertices(
    command: UiNativeGlyphCommand,
    target_extent: [u32; 2],
) -> [super::super::GlyphVertex; 6] {
    let [x, y, width, height] = command.target;
    let left = x * 2.0 / target_extent[0] as f32 - 1.0;
    let right = (x + width) * 2.0 / target_extent[0] as f32 - 1.0;
    let top = 1.0 - y * 2.0 / target_extent[1] as f32;
    let bottom = 1.0 - (y + height) * 2.0 / target_extent[1] as f32;
    let [u, v, uv_width, uv_height] = command.texture_uv;
    let far_u = u + uv_width;
    let far_v = v + uv_height;
    let foreground = command.foreground.channels();
    let color = [
        linear_channel(foreground[0]),
        linear_channel(foreground[1]),
        linear_channel(foreground[2]),
        f32::from(foreground[3]) / 255.0 * command.opacity,
    ];
    [
        glyph_vertex(left, bottom, u, far_v, color),
        glyph_vertex(right, bottom, far_u, far_v, color),
        glyph_vertex(left, top, u, v, color),
        glyph_vertex(left, top, u, v, color),
        glyph_vertex(right, bottom, far_u, far_v, color),
        glyph_vertex(right, top, far_u, v, color),
    ]
}

fn glyph_vertex(x: f32, y: f32, u: f32, v: f32, color: [f32; 4]) -> super::super::GlyphVertex {
    super::super::GlyphVertex {
        position: [x, y],
        texture_uv: [u, v],
        color,
    }
}

fn linear_channel(channel: u8) -> f32 {
    let encoded = f32::from(channel) / 255.0;
    if encoded <= 0.040_45 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}
