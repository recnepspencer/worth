//! Rasterize one joined retained render item without changing semantic order.
use super::render_order::UiNativeRetainedRenderItem;
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::appearance::UiNativeAppearanceCommand;
use crate::native::presentation::raster::{raster_damage_for_basis, UiNativeRasterBasis};
use crate::native::presentation::{RasterRect, UiNativeRasterOperation};
use worth_ui_host_contract::{
    UiMountedPaintCommand, UiMountedPaintCommandIdentity, UiMountedPresentationSampleChange,
};

impl UiNativeRetainedDrawList {
    pub(super) fn raster_render_items(
        &self,
        items: &[UiNativeRetainedRenderItem],
        basis: UiNativeRasterBasis,
        atlas: &crate::native::text_atlas::UiNativeTextAtlas,
        clear: Option<RasterRect>,
    ) -> Result<Vec<UiNativeRasterOperation>, Denial> {
        let mut result = Vec::new();
        for item in items {
            let operations = match item {
                UiNativeRetainedRenderItem::Appearance(key) => {
                    let command = self
                        .staged_appearance
                        .as_ref()
                        .and_then(|(_, retained)| retained.command(*key))
                        .cloned()
                        .ok_or(Denial::CommandMismatch)?;
                    self.appearance_command_operations(command, basis)?
                }
                UiNativeRetainedRenderItem::Paint(identity) => {
                    self.ordinary_command_operations(*identity, basis, atlas)?
                }
            };
            result.extend(operations.into_iter().filter_map(|operation| match clear {
                Some(clear) => clip_operation(operation, clear, basis),
                None => Some(operation),
            }));
        }
        Ok(result)
    }

    fn ordinary_command_operations(
        &self,
        identity: UiMountedPaintCommandIdentity,
        basis: UiNativeRasterBasis,
        atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    ) -> Result<Vec<UiNativeRasterOperation>, Denial> {
        let command = self.command(identity).ok_or(Denial::CommandMismatch)?;
        let sample = self.sample_override(identity);
        let opacity = sample.map_or(1.0, |sample| sample.opacity().factor());
        let operation = match command {
            UiMountedPaintCommand::FilledRect { mechanic, .. } => {
                let Some(bounds) = super::sampled_visible_bounds(command, sample)
                    .map_err(|_| Denial::CommandMismatch)?
                else {
                    return Ok(Vec::new());
                };
                raster_damage_for_basis(bounds, basis)
                    .map_err(|_| Denial::CommandMismatch)?
                    .map(|rect| UiNativeRasterOperation::FilledRect {
                        rect,
                        source_rgba8: crate::native::presentation::retained_raster::sampled_color(
                            mechanic.color().channels(),
                            opacity,
                        ),
                    })
            }
            UiMountedPaintCommand::PortalOverlay { mechanic, .. } => {
                let Some(bounds) = super::sampled_visible_bounds(command, sample)
                    .map_err(|_| Denial::CommandMismatch)?
                else {
                    return Ok(Vec::new());
                };
                raster_damage_for_basis(bounds, basis)
                    .map_err(|_| Denial::CommandMismatch)?
                    .map(|rect| UiNativeRasterOperation::FilledRect {
                        rect,
                        source_rgba8: crate::native::presentation::retained_raster::sampled_color(
                            mechanic.color().channels(),
                            opacity,
                        ),
                    })
            }
            UiMountedPaintCommand::SemanticText { .. } => {
                return self
                    .plan_text_commands(identity, atlas, basis)
                    .map(|glyphs| {
                        glyphs
                            .iter()
                            .copied()
                            .map(UiNativeRasterOperation::Glyph)
                            .collect()
                    })
            }
        };
        Ok(operation.into_iter().collect())
    }

    fn appearance_command_operations(
        &self,
        command: UiNativeAppearanceCommand,
        basis: UiNativeRasterBasis,
    ) -> Result<Vec<UiNativeRasterOperation>, Denial> {
        use crate::native::presentation::appearance::{
            UiNativeBackdropPipeline, UiNativeOutlinePipeline, UiNativeSurfacePipeline,
        };
        let scale = self
            .staged_appearance
            .as_ref()
            .ok_or(Denial::CommandMismatch)?
            .1
            .scale();
        let surface_operation =
            |primitive: crate::native::presentation::appearance::UiNativeSurfacePrimitive,
             sample: Option<UiMountedPresentationSampleChange>|
             -> Result<Option<UiNativeRasterOperation>, Denial> {
                let operation = match sample {
                    Some(sample) => primitive
                        .raster_operation_with_sample(sample, basis)
                        .map_err(|_| Denial::CommandMismatch)?,
                    None => primitive
                        .raster_operation(basis.extent())
                        .map_err(|_| Denial::CommandMismatch)?,
                };
                Ok(operation.map(UiNativeRasterOperation::Surface))
            };
        let operation = match command {
            UiNativeAppearanceCommand::Surface(mechanic) => surface_operation(
                UiNativeSurfacePipeline::prepare(&mechanic, scale)
                    .map_err(|_| Denial::CommandMismatch)?,
                self.appearance_sample(mechanic.node_receipt().mounted_instance(), false)?,
            )?,
            UiNativeAppearanceCommand::PortalSurface(mechanic) => surface_operation(
                UiNativeSurfacePipeline::prepare(mechanic.surface(), scale)
                    .map_err(|_| Denial::CommandMismatch)?,
                self.appearance_sample(mechanic.portal_instance(), true)?,
            )?,
            UiNativeAppearanceCommand::Outline(mechanic) => {
                let outline = UiNativeOutlinePipeline::prepare(&mechanic, scale)
                    .map_err(|_| Denial::CommandMismatch)?;
                surface_operation(
                    UiNativeSurfacePipeline::prepare_outline(&outline),
                    self.appearance_sample(mechanic.node_receipt().mounted_instance(), false)?,
                )?
            }
            UiNativeAppearanceCommand::Backdrop(mechanic) => {
                let primitive = UiNativeBackdropPipeline::prepare(&mechanic, scale)
                    .map_err(|_| Denial::CommandMismatch)?;
                let bounds = primitive.extent().pixel_bounds();
                let clip = primitive.clip();
                let [width, height] = basis.extent();
                let edges = [
                    bounds.left.max(clip.left).clamp(0, i64::from(width)) as u32,
                    bounds.top.max(clip.top).clamp(0, i64::from(height)) as u32,
                    bounds.right.min(clip.right).clamp(0, i64::from(width)) as u32,
                    bounds.bottom.min(clip.bottom).clamp(0, i64::from(height)) as u32,
                ];
                crate::native::presentation::raster::raster_physical_bounds(edges, basis.extent())
                    .map(|rect| UiNativeRasterOperation::FilledRect {
                        rect,
                        source_rgba8: crate::native::presentation::retained_raster::sampled_color(
                            primitive.background().straight_srgba(),
                            f32::from(primitive.opacity()) / f32::from(u16::MAX),
                        ),
                    })
            }
            UiNativeAppearanceCommand::TextForeground(_)
            | UiNativeAppearanceCommand::OverlayOrder(_)
            | UiNativeAppearanceCommand::PointerAffordance(_) => None,
        };
        Ok(operation.into_iter().collect())
    }

    fn appearance_sample(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        portal: bool,
    ) -> Result<Option<UiMountedPresentationSampleChange>, Denial> {
        let samples = self
            .commands
            .identities_for_instance(instance)
            .filter_map(|identity| {
                let command = self.command(identity)?;
                let matches_family = matches!(command, UiMountedPaintCommand::PortalOverlay { .. });
                (matches_family == portal)
                    .then(|| self.sample_override(identity))
                    .flatten()
            })
            .collect::<Vec<_>>();
        let mut selected = None;
        for sample in samples {
            if selected.is_some_and(|current: UiMountedPresentationSampleChange| {
                current.transform() != sample.transform() || current.opacity() != sample.opacity()
            }) {
                return Err(Denial::CommandMismatch);
            }
            selected = Some(sample);
        }
        Ok(selected)
    }
}

fn clip_operation(
    operation: UiNativeRasterOperation,
    clear: RasterRect,
    basis: UiNativeRasterBasis,
) -> Option<UiNativeRasterOperation> {
    match operation {
        UiNativeRasterOperation::Surface(surface) => surface
            .clipped_to(clear, basis.extent())
            .map(UiNativeRasterOperation::Surface),
        UiNativeRasterOperation::FilledRect { rect, source_rgba8 } => rect
            .intersection(clear, basis.extent())
            .map(|rect| UiNativeRasterOperation::FilledRect { rect, source_rgba8 }),
        UiNativeRasterOperation::Glyph(glyph) => {
            crate::native::presentation::text::clip_glyph_command(glyph, clear.physical_bounds())
                .map(UiNativeRasterOperation::Glyph)
        }
        UiNativeRasterOperation::Clear(_) => None,
    }
}
