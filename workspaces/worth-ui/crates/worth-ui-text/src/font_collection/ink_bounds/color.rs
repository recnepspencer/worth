use skrifa::{
    color::{Brush, ColorPainter, CompositeMode, PaintError, Transform},
    instance::{LocationRef, NormalizedCoord},
    raw::types::BoundingBox,
    GlyphId, MetadataProvider,
};

pub(super) use kurbo::Rect;

use crate::font_collection::{color_glyph::path, UiFontGlyphInkBounds};

use super::color_region::{Coverage, Layer, Region};

pub(super) fn bounds(
    font: &harfrust::FontRef<'_>,
    coords: &[NormalizedCoord],
    glyph_id: GlyphId,
) -> Option<Option<UiFontGlyphInkBounds>> {
    let glyph = font.color_glyphs().get(glyph_id)?;
    let mut painter = InkPainter::new(font, coords);
    glyph.paint(LocationRef::new(coords), &mut painter).ok()?;
    if painter.failed {
        return None;
    }
    Some(painter.finish())
}

pub(super) struct InkPainter<'font, 'data, 'coords> {
    font: &'font harfrust::FontRef<'data>,
    coords: &'coords [NormalizedCoord],
    transform: Affine,
    transforms: Vec<Affine>,
    clip: Option<Region>,
    clips: Vec<Option<Region>>,
    painted: Coverage,
    layers: Vec<Layer>,
    palette_alphas: Box<[u8]>,
    failed: bool,
}

impl<'font, 'data, 'coords> InkPainter<'font, 'data, 'coords> {
    pub(super) fn new(
        font: &'font harfrust::FontRef<'data>,
        coords: &'coords [NormalizedCoord],
    ) -> Self {
        Self {
            font,
            coords,
            transform: Affine::IDENTITY,
            transforms: Vec::new(),
            clip: None,
            clips: Vec::new(),
            painted: Coverage::default(),
            layers: Vec::new(),
            palette_alphas: font
                .color_palettes()
                .get(0)
                .map(|palette| {
                    palette
                        .colors()
                        .iter()
                        .map(|color| color.alpha())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
                .into_boxed_slice(),
            failed: false,
        }
    }

    pub(super) fn push_clip_rect(&mut self, rect: Rect) {
        let path = path::rectangle(rect, self.transform.to_kurbo());
        self.push_clip_path(path);
    }

    pub(super) fn push_clip_path(&mut self, path: kurbo::BezPath) {
        self.clips.push(self.clip.clone());
        let next = Region::new(path);
        self.clip = match self.clip.take() {
            Some(current) => match current.intersect(next) {
                Ok(intersection) => Some(intersection),
                Err(_) => {
                    self.failed = true;
                    None
                }
            },
            None => Some(next),
        };
    }

    pub(super) fn finish(self) -> Option<UiFontGlyphInkBounds> {
        let rect = self.painted.bounds()?;
        Some(UiFontGlyphInkBounds {
            x_min: rect.x0.floor() as i32,
            y_min: rect.y0.floor() as i32,
            x_max: rect.x1.ceil() as i32,
            y_max: rect.y1.ceil() as i32,
        })
    }
}

impl ColorPainter for InkPainter<'_, '_, '_> {
    fn push_transform(&mut self, transform: Transform) {
        self.transforms.push(self.transform);
        self.transform = self.transform.concat(Affine::from(transform));
    }

    fn pop_transform(&mut self) {
        self.transform = self.transforms.pop().unwrap_or(Affine::IDENTITY);
    }

    fn push_clip_glyph(&mut self, glyph_id: GlyphId) {
        let path = path::glyph(self.font, self.coords, glyph_id, self.transform.to_kurbo())
            .unwrap_or_default();
        self.push_clip_path(path);
    }

    fn push_clip_box(&mut self, clip_box: BoundingBox<f32>) {
        self.push_clip_rect(Rect {
            x0: f64::from(clip_box.x_min),
            y0: f64::from(clip_box.y_min),
            x1: f64::from(clip_box.x_max),
            y1: f64::from(clip_box.y_max),
        });
    }

    fn pop_clip(&mut self) {
        self.clip = self.clips.pop().unwrap_or(None);
    }

    fn fill(&mut self, brush: Brush<'_>) {
        if let (Some(clip), Some(alpha)) = (
            self.clip.clone(),
            brush_alpha_range(&brush, &self.palette_alphas),
        ) && self
            .painted
            .insert(clip.path(), alpha.minimum >= 1.0)
            .is_err()
        {
            self.failed = true;
        }
    }

    fn push_layer(&mut self, composite_mode: CompositeMode) {
        self.layers.push(Layer {
            backdrop: core::mem::take(&mut self.painted),
            composite_mode,
        });
    }

    fn pop_layer(&mut self) {
        let Some(layer) = self.layers.pop() else {
            return;
        };
        match Coverage::composite(self.painted.clone(), layer.backdrop, layer.composite_mode) {
            Ok(painted) => self.painted = painted,
            Err(_) => self.failed = true,
        }
    }

    fn paint_cached_color_glyph(
        &mut self,
        _glyph: GlyphId,
    ) -> Result<skrifa::color::PaintCachedColorGlyph, PaintError> {
        Ok(skrifa::color::PaintCachedColorGlyph::Unimplemented)
    }
}

fn brush_alpha_range(brush: &Brush<'_>, palette_alphas: &[u8]) -> Option<AlphaRange> {
    let color_alpha = |palette_index: u16, alpha: f32| {
        let palette = palette_alphas
            .get(usize::from(palette_index))
            .map_or(0.0, |alpha| f32::from(*alpha) / 255.0);
        alpha.clamp(0.0, 1.0) * palette
    };
    let range = match brush {
        Brush::Solid {
            palette_index,
            alpha,
        } => {
            let alpha = color_alpha(*palette_index, *alpha);
            AlphaRange {
                minimum: alpha,
                maximum: alpha,
            }
        }
        Brush::LinearGradient { color_stops, .. }
        | Brush::RadialGradient { color_stops, .. }
        | Brush::SweepGradient { color_stops, .. } => color_stops.iter().fold(
            AlphaRange {
                minimum: 1.0,
                maximum: 0.0,
            },
            |range, stop| {
                let alpha = color_alpha(stop.palette_index, stop.alpha);
                AlphaRange {
                    minimum: range.minimum.min(alpha),
                    maximum: range.maximum.max(alpha),
                }
            },
        ),
    };
    (range.maximum > 0.0).then_some(range)
}

#[derive(Clone, Copy)]
struct AlphaRange {
    minimum: f32,
    maximum: f32,
}

#[derive(Clone, Copy)]
struct Affine {
    xx: f32,
    yx: f32,
    xy: f32,
    yy: f32,
    dx: f32,
    dy: f32,
}

impl Affine {
    const IDENTITY: Self = Self {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: 1.0,
        dx: 0.0,
        dy: 0.0,
    };

    fn concat(self, next: Self) -> Self {
        Self {
            xx: self.xx * next.xx + self.xy * next.yx,
            yx: self.yx * next.xx + self.yy * next.yx,
            xy: self.xx * next.xy + self.xy * next.yy,
            yy: self.yx * next.xy + self.yy * next.yy,
            dx: self.xx * next.dx + self.xy * next.dy + self.dx,
            dy: self.yx * next.dx + self.yy * next.dy + self.dy,
        }
    }

    fn to_kurbo(self) -> kurbo::Affine {
        kurbo::Affine::new([
            f64::from(self.xx),
            f64::from(self.yx),
            f64::from(self.xy),
            f64::from(self.yy),
            f64::from(self.dx),
            f64::from(self.dy),
        ])
    }
}

impl From<Transform> for Affine {
    fn from(value: Transform) -> Self {
        Self {
            xx: value.xx,
            yx: value.yx,
            xy: value.xy,
            yy: value.yy,
            dx: value.dx,
            dy: value.dy,
        }
    }
}
