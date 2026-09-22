use std::io::Cursor;

use read_fonts::{
    tables::{
        bitmap::{BitmapContent, BitmapData, BitmapMetrics, BitmapSize},
        cbdt::Cbdt,
        cblc::Cblc,
    },
    types::GlyphId,
    TableProvider,
};
use skrifa::bitmap::{BitmapData as SkrifaBitmapData, Origin};

use crate::font_collection::color_glyph::bitmap_selection::{
    selections, UiBitmapSelection, UiCbdtCompositeSelection,
};
use crate::font_collection::UiFontGlyphInkBounds;

pub(super) fn bounds(
    font: &harfrust::FontRef<'_>,
    glyph_id: GlyphId,
) -> Option<Option<UiFontGlyphInkBounds>> {
    let selections = selections(font, glyph_id).ok()?;
    if selections.is_empty() {
        return None;
    }
    let mut bounds = None;
    for selection in selections {
        if let Some(selection_bounds) = selection_bounds(font, &selection)? {
            bounds = Some(union_font_bounds(bounds, selection_bounds));
        }
    }
    Some(bounds)
}

fn selection_bounds(
    font: &harfrust::FontRef<'_>,
    selection: &UiBitmapSelection<'_>,
) -> Option<Option<UiFontGlyphInkBounds>> {
    let placement = BitmapPlacement::from_selection(selection)?;
    let pixels = match selection {
        UiBitmapSelection::Direct(glyph) => alpha_bounds(&glyph.data, glyph.width, glyph.height)?,
        UiBitmapSelection::CbdtComposite(composite) => composite_alpha_bounds(composite)?,
    };
    let Some(pixels) = pixels else {
        return Some(None);
    };
    let units_per_em = f32::from(font.head().ok()?.units_per_em());
    let scale_x = units_per_em / placement.ppem_x;
    let scale_y = units_per_em / placement.ppem_y;
    if !scale_x.is_finite() || !scale_y.is_finite() {
        return Some(None);
    }
    let image_left = placement.bearing_x + placement.inner_bearing_x * scale_x;
    let image_top = match placement.placement_origin {
        Origin::TopLeft => placement.bearing_y + placement.inner_bearing_y * scale_y,
        Origin::BottomLeft => {
            placement.bearing_y
                + placement.inner_bearing_y * scale_y
                + placement.height as f32 * scale_y
        }
    };
    Some(Some(UiFontGlyphInkBounds {
        x_min: (image_left + pixels.left as f32 * scale_x).floor() as i32,
        y_min: (image_top - pixels.bottom as f32 * scale_y).floor() as i32,
        x_max: (image_left + pixels.right as f32 * scale_x).ceil() as i32,
        y_max: (image_top - pixels.top as f32 * scale_y).ceil() as i32,
    }))
}

fn union_font_bounds(
    current: Option<UiFontGlyphInkBounds>,
    candidate: UiFontGlyphInkBounds,
) -> UiFontGlyphInkBounds {
    current.map_or(candidate, |current| UiFontGlyphInkBounds {
        x_min: current.x_min.min(candidate.x_min),
        y_min: current.y_min.min(candidate.y_min),
        x_max: current.x_max.max(candidate.x_max),
        y_max: current.y_max.max(candidate.y_max),
    })
}

struct BitmapPlacement {
    height: u32,
    ppem_x: f32,
    ppem_y: f32,
    bearing_x: f32,
    bearing_y: f32,
    inner_bearing_x: f32,
    inner_bearing_y: f32,
    placement_origin: Origin,
}

impl BitmapPlacement {
    fn from_selection(selection: &UiBitmapSelection<'_>) -> Option<Self> {
        match selection {
            UiBitmapSelection::Direct(glyph) => Some(Self {
                height: glyph.height,
                ppem_x: glyph.ppem_x,
                ppem_y: glyph.ppem_y,
                bearing_x: glyph.bearing_x,
                bearing_y: glyph.bearing_y,
                inner_bearing_x: glyph.inner_bearing_x,
                inner_bearing_y: glyph.inner_bearing_y,
                placement_origin: glyph.placement_origin,
            }),
            UiBitmapSelection::CbdtComposite(composite) => {
                let (_, height, bearing_x, bearing_y) = raw_metrics(&composite.data)?;
                Some(Self {
                    height,
                    ppem_x: f32::from(composite.size.ppem_x()),
                    ppem_y: f32::from(composite.size.ppem_y()),
                    bearing_x: 0.0,
                    bearing_y: 0.0,
                    inner_bearing_x: bearing_x,
                    inner_bearing_y: bearing_y,
                    placement_origin: Origin::TopLeft,
                })
            }
        }
    }
}

fn raw_metrics(data: &BitmapData<'_>) -> Option<(u32, u32, f32, f32)> {
    match data.metrics {
        BitmapMetrics::Small(metrics) => Some((
            u32::from(metrics.width()),
            u32::from(metrics.height()),
            f32::from(metrics.bearing_x()),
            f32::from(metrics.bearing_y()),
        )),
        BitmapMetrics::Big(metrics) => Some((
            u32::from(metrics.width()),
            u32::from(metrics.height()),
            f32::from(metrics.hori_bearing_x()),
            f32::from(metrics.hori_bearing_y()),
        )),
    }
}

struct CompositeAlphaContext<'font, 'borrow> {
    cblc: &'borrow Cblc<'font>,
    cbdt: &'borrow Cbdt<'font>,
    size: &'borrow BitmapSize,
}

fn composite_alpha_bounds(selection: &UiCbdtCompositeSelection<'_>) -> Option<Option<PixelBounds>> {
    let context = CompositeAlphaContext {
        cblc: &selection.cblc,
        cbdt: &selection.cbdt,
        size: &selection.size,
    };
    composite_data_alpha(&context, selection.glyph, &selection.data, &mut Vec::new())
}

fn composite_data_alpha<'font, 'borrow>(
    context: &CompositeAlphaContext<'font, 'borrow>,
    glyph: GlyphId,
    data: &BitmapData<'font>,
    stack: &mut Vec<u32>,
) -> Option<Option<PixelBounds>> {
    let (width, height, _, _) = raw_metrics(data)?;
    match &data.content {
        BitmapContent::Data(format, bytes) => {
            let alpha = match format {
                read_fonts::tables::bitmap::BitmapDataFormat::Png => {
                    png_alpha(bytes, width, height)?
                }
                read_fonts::tables::bitmap::BitmapDataFormat::ByteAligned
                    if context.size.bit_depth() == 32 =>
                {
                    bgra_alpha(bytes, width, height)?
                }
                _ => return None,
            };
            Some(nonzero_bounds(&alpha, width, height))
        }
        BitmapContent::Composite(components) => {
            if components.is_empty() || stack.len() >= 64 || stack.contains(&glyph.to_u32()) {
                return None;
            }
            stack.push(glyph.to_u32());
            let mut bounds = None;
            for component in *components {
                let child_id = GlyphId::from(component.glyph_id());
                let location = context
                    .size
                    .location(context.cblc.offset_data(), child_id)
                    .ok()?;
                if location.is_empty() {
                    return None;
                }
                let child = context.cbdt.data(&location).ok()?;
                let child_bounds = composite_data_alpha(context, child_id, &child, stack)?;
                if let Some(child_bounds) = child_bounds {
                    bounds = union_placed_bounds(
                        bounds,
                        child_bounds,
                        CompositePlacement {
                            offset_x: i32::from(component.x_offset()),
                            offset_y: i32::from(component.y_offset()),
                            width,
                            height,
                        },
                    );
                }
            }
            stack.pop();
            Some(bounds)
        }
    }
}

pub(super) fn union_placed_bounds(
    current: Option<PixelBounds>,
    child: PixelBounds,
    placement: CompositePlacement,
) -> Option<PixelBounds> {
    let left = (i64::from(placement.offset_x) + i64::from(child.left)).max(0);
    let top = (i64::from(placement.offset_y) + i64::from(child.top)).max(0);
    let right =
        (i64::from(placement.offset_x) + i64::from(child.right)).min(i64::from(placement.width));
    let bottom =
        (i64::from(placement.offset_y) + i64::from(child.bottom)).min(i64::from(placement.height));
    if left >= right || top >= bottom {
        return current;
    }
    let placed = PixelBounds {
        left: u32::try_from(left).ok()?,
        top: u32::try_from(top).ok()?,
        right: u32::try_from(right).ok()?,
        bottom: u32::try_from(bottom).ok()?,
    };
    Some(match current {
        Some(current) => PixelBounds {
            left: current.left.min(placed.left),
            top: current.top.min(placed.top),
            right: current.right.max(placed.right),
            bottom: current.bottom.max(placed.bottom),
        },
        None => placed,
    })
}

#[derive(Clone, Copy)]
pub(super) struct CompositePlacement {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) offset_x: i32,
    pub(super) offset_y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PixelBounds {
    pub(super) left: u32,
    pub(super) top: u32,
    pub(super) right: u32,
    pub(super) bottom: u32,
}

pub(super) fn alpha_bounds(
    data: &SkrifaBitmapData<'_>,
    width: u32,
    height: u32,
) -> Option<Option<PixelBounds>> {
    let alpha = match data {
        SkrifaBitmapData::Bgra(bytes) => bgra_alpha(bytes, width, height)?,
        SkrifaBitmapData::Mask(mask) => mask.decode(width, height).ok()?,
        SkrifaBitmapData::Png(bytes) => png_alpha(bytes, width, height)?,
    };
    Some(nonzero_bounds(&alpha, width, height))
}

fn bgra_alpha(bytes: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    let pixels = usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?;
    let expected = pixels.checked_mul(4)?;
    (bytes.len() >= expected).then(|| {
        bytes[..expected]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[3])
            .collect()
    })
}

fn png_alpha(bytes: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    let mut decoder = png::Decoder::new_with_limits(
        Cursor::new(bytes),
        png::Limits {
            bytes: 2 * 1024 * 1024,
        },
    );
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let output_size = reader.output_buffer_size()?;
    let mut output = vec![0; output_size];
    let info = reader.next_frame(&mut output).ok()?;
    if info.width != width || info.height != height {
        return None;
    }
    let samples = info.color_type.samples();
    let alpha_index = match info.color_type {
        png::ColorType::GrayscaleAlpha => Some(1),
        png::ColorType::Rgba => Some(3),
        png::ColorType::Grayscale | png::ColorType::Rgb => None,
        png::ColorType::Indexed => return None,
    };
    let pixels = usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?;
    match alpha_index {
        Some(alpha_index) => Some(
            output[..info.buffer_size()]
                .chunks_exact(samples)
                .take(pixels)
                .map(|pixel| pixel[alpha_index])
                .collect(),
        ),
        None => Some(vec![255; pixels]),
    }
}

fn nonzero_bounds(alpha: &[u8], width: u32, height: u32) -> Option<PixelBounds> {
    let mut bounds: Option<PixelBounds> = None;
    for (index, value) in alpha.iter().copied().enumerate() {
        if value == 0 {
            continue;
        }
        let index = u32::try_from(index).ok()?;
        let x = index % width;
        let y = index / width;
        if y >= height {
            break;
        }
        bounds = Some(match bounds {
            Some(bounds) => PixelBounds {
                left: bounds.left.min(x),
                top: bounds.top.min(y),
                right: bounds.right.max(x + 1),
                bottom: bounds.bottom.max(y + 1),
            },
            None => PixelBounds {
                left: x,
                top: y,
                right: x + 1,
                bottom: y + 1,
            },
        });
    }
    bounds
}
