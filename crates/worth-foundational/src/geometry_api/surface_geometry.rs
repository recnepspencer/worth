use super::UiAppearanceLogicalLength;
use super::{NormalizedPoint, VectorPath, VectorPathSegment};

/// Content geometry only. Color, opacity and ordering remain appearance-owned.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum UiSurfaceGeometry {
    #[default]
    RoundedRectangle,
    Vector(UiVectorSurfaceGeometry),
    SoftShadow(UiSoftShadowGeometry),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiVectorSurfaceGeometry {
    path: VectorPath,
    stroke_width: Option<UiAppearanceLogicalLength>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiSoftShadowGeometry {
    sigma: UiAppearanceLogicalLength,
    radius: UiAppearanceLogicalLength,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSurfaceGeometryDenial {
    OpenFillContour,
    StrokeWidthOutOfRange,
    ShadowSigmaOutOfRange,
}

impl UiSurfaceGeometry {
    /// Versioned geometry fingerprint; equality remains the reuse authority.
    pub fn semantic_digest(&self) -> u64 {
        let mut digest = 0x7375726667656f01_u64;
        let mut fold = |value: u64| {
            digest = digest.rotate_left(11) ^ value;
        };
        match self {
            Self::RoundedRectangle => fold(0),
            Self::SoftShadow(shadow) => {
                fold(1);
                fold(u64::from(shadow.sigma().subpixels()));
                fold(u64::from(shadow.radius().subpixels()));
            }
            Self::Vector(vector) => {
                fold(2);
                fold(u64::from(
                    vector.stroke_width().map_or(0, |w| w.subpixels()),
                ));
                for segment in vector.path().segments() {
                    let point = |p: NormalizedPoint| {
                        let [x, y] = p.coordinates();
                        (u64::from(x) << 16) | u64::from(y)
                    };
                    match *segment {
                        VectorPathSegment::MoveTo(p) => {
                            fold(1);
                            fold(point(p));
                        }
                        VectorPathSegment::LineTo(p) => {
                            fold(2);
                            fold(point(p));
                        }
                        VectorPathSegment::CubicTo {
                            control_a,
                            control_b,
                            end,
                        } => {
                            fold(3);
                            fold(point(control_a));
                            fold(point(control_b));
                            fold(point(end));
                        }
                        VectorPathSegment::Close => fold(4),
                    }
                }
            }
        }
        digest
    }
}

impl UiVectorSurfaceGeometry {
    pub fn fill(path: VectorPath) -> Result<Self, UiSurfaceGeometryDenial> {
        if !path.all_contours_closed() {
            return Err(UiSurfaceGeometryDenial::OpenFillContour);
        }
        Ok(Self {
            path,
            stroke_width: None,
        })
    }

    /// Round caps and joins. The path view box is inset by half the stroke width.
    pub fn stroke(
        path: VectorPath,
        width: UiAppearanceLogicalLength,
    ) -> Result<Self, UiSurfaceGeometryDenial> {
        if width.subpixels() == 0 || width.subpixels() > 32_000 {
            return Err(UiSurfaceGeometryDenial::StrokeWidthOutOfRange);
        }
        Ok(Self {
            path,
            stroke_width: Some(width),
        })
    }

    pub fn path(&self) -> &VectorPath {
        &self.path
    }
    pub const fn stroke_width(&self) -> Option<UiAppearanceLogicalLength> {
        self.stroke_width
    }
}

impl UiSoftShadowGeometry {
    /// The allocation includes a three-sigma margin around the rounded caster.
    /// Finite support ensures damage and clipping include every painted pixel.
    pub fn new(
        sigma: UiAppearanceLogicalLength,
        radius: UiAppearanceLogicalLength,
    ) -> Result<Self, UiSurfaceGeometryDenial> {
        if sigma.subpixels() == 0 || sigma.subpixels() > 128_000 {
            return Err(UiSurfaceGeometryDenial::ShadowSigmaOutOfRange);
        }
        Ok(Self { sigma, radius })
    }

    pub const fn sigma(self) -> UiAppearanceLogicalLength {
        self.sigma
    }
    pub const fn radius(self) -> UiAppearanceLogicalLength {
        self.radius
    }
}
