use std::sync::Arc;

/// Coordinates in ten-thousandths of a declared view box, including its edges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NormalizedPoint {
    x: u16,
    y: u16,
}

impl NormalizedPoint {
    pub const SCALE: u16 = 10_000;

    pub const fn new(x: u16, y: u16) -> Result<Self, VectorPathDenial> {
        if x > Self::SCALE || y > Self::SCALE {
            return Err(VectorPathDenial::PointOutsideViewBox);
        }
        Ok(Self { x, y })
    }

    pub const fn coordinates(self) -> [u16; 2] {
        [self.x, self.y]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorPathSegment {
    MoveTo(NormalizedPoint),
    LineTo(NormalizedPoint),
    CubicTo {
        control_a: NormalizedPoint,
        control_b: NormalizedPoint,
        end: NormalizedPoint,
    },
    Close,
}

/// A validated sequence; closed contours use the even-odd fill rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorPath {
    segments: Arc<[VectorPathSegment]>,
    all_contours_closed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorPathDenial {
    PointOutsideViewBox,
    Empty,
    CapacityExceeded,
    MissingContourStart,
    EmptyContour,
}

impl VectorPath {
    pub const MAX_SEGMENTS: usize = 256;

    pub fn new(segments: Vec<VectorPathSegment>) -> Result<Self, VectorPathDenial> {
        if segments.is_empty() {
            return Err(VectorPathDenial::Empty);
        }
        if segments.len() > Self::MAX_SEGMENTS {
            return Err(VectorPathDenial::CapacityExceeded);
        }
        let mut active = false;
        let mut drawn = false;
        let mut all_closed = true;
        for segment in &segments {
            match segment {
                VectorPathSegment::MoveTo(_) => {
                    if active && !drawn {
                        return Err(VectorPathDenial::EmptyContour);
                    }
                    all_closed &= !active;
                    active = true;
                    drawn = false;
                }
                VectorPathSegment::LineTo(_) | VectorPathSegment::CubicTo { .. } => {
                    if !active {
                        return Err(VectorPathDenial::MissingContourStart);
                    }
                    drawn = true;
                }
                VectorPathSegment::Close => {
                    if !active {
                        return Err(VectorPathDenial::MissingContourStart);
                    }
                    if !drawn {
                        return Err(VectorPathDenial::EmptyContour);
                    }
                    active = false;
                }
            }
        }
        if active && !drawn {
            return Err(VectorPathDenial::EmptyContour);
        }
        Ok(Self {
            segments: segments.into(),
            all_contours_closed: all_closed && !active,
        })
    }

    pub fn segments(&self) -> &[VectorPathSegment] {
        &self.segments
    }

    pub const fn all_contours_closed(&self) -> bool {
        self.all_contours_closed
    }
}
