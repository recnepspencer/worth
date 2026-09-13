#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedCoordinateSpace {
    Viewport,
    Window,
    GraphNodeLocal,
    HostSurface,
    PortalLayer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedGeometryPosture {
    Area,
    Empty,
    Offscreen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedGeometryDenial {
    NonFinite,
    NegativeExtent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedTransformProjection {
    Identity,
    Omitted(super::UiMountedOmissionReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedAllocationBasis {
    receipt_identity: u64,
    receipt_generation: u64,
    coordinate_ownership: u64,
    transform: UiMountedTransformProjection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedCanonicalBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    coordinate_space: UiMountedCoordinateSpace,
    posture: UiMountedGeometryPosture,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedCanonicalBoxInput {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub coordinate_space: UiMountedCoordinateSpace,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiMountedAllocationProjection {
    Known {
        bounds: UiMountedCanonicalBox,
        basis: UiMountedAllocationBasis,
    },
    PortalAnchorObservation {
        bounds: UiMountedCanonicalBox,
        basis: UiMountedAllocationBasis,
    },
    Omitted(super::UiMountedOmissionReason),
}

impl UiMountedCanonicalBox {
    pub fn canonicalize(
        input: UiMountedCanonicalBoxInput,
    ) -> Result<Self, UiMountedGeometryDenial> {
        if [input.x, input.y, input.width, input.height]
            .iter()
            .any(|value| !value.is_finite())
        {
            return Err(UiMountedGeometryDenial::NonFinite);
        }
        if input.width < 0.0 || input.height < 0.0 {
            return Err(UiMountedGeometryDenial::NegativeExtent);
        }
        let posture = canonical_posture(input)?;
        Ok(Self {
            x: input.x,
            y: input.y,
            width: input.width,
            height: input.height,
            coordinate_space: input.coordinate_space,
            posture,
        })
    }

    pub fn x(self) -> f32 {
        self.x
    }
    pub fn y(self) -> f32 {
        self.y
    }
    pub fn width(self) -> f32 {
        self.width
    }
    pub fn height(self) -> f32 {
        self.height
    }
    /// The overlap of two boxes in one coordinate space; `None` when they do
    /// not overlap or their spaces differ.
    pub fn intersection(self, other: Self) -> Option<Self> {
        if self.coordinate_space() != other.coordinate_space() {
            return None;
        }
        let left = self.x().max(other.x());
        let top = self.y().max(other.y());
        let right = (self.x() + self.width()).min(other.x() + other.width());
        let bottom = (self.y() + self.height()).min(other.y() + other.height());
        if right <= left || bottom <= top {
            return None;
        }
        Self::canonicalize(UiMountedCanonicalBoxInput {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
            coordinate_space: self.coordinate_space(),
        })
        .ok()
    }

    pub fn coordinate_space(self) -> UiMountedCoordinateSpace {
        self.coordinate_space
    }
    pub fn posture(self) -> UiMountedGeometryPosture {
        self.posture
    }
}

impl UiMountedAllocationBasis {
    pub fn new(
        receipt_identity: u64,
        receipt_generation: u64,
        coordinate_ownership: u64,
        transform: UiMountedTransformProjection,
    ) -> Self {
        Self {
            receipt_identity,
            receipt_generation,
            coordinate_ownership,
            transform,
        }
    }

    pub fn receipt_identity(self) -> u64 {
        self.receipt_identity
    }
    pub fn receipt_generation(self) -> u64 {
        self.receipt_generation
    }
    pub fn coordinate_ownership(self) -> u64 {
        self.coordinate_ownership
    }
    pub fn transform(self) -> UiMountedTransformProjection {
        self.transform
    }
}

fn canonical_posture(
    input: UiMountedCanonicalBoxInput,
) -> Result<UiMountedGeometryPosture, UiMountedGeometryDenial> {
    if input.width == 0.0 || input.height == 0.0 {
        return Ok(UiMountedGeometryPosture::Empty);
    }
    let right = input.x + input.width;
    let bottom = input.y + input.height;
    if !right.is_finite() || !bottom.is_finite() {
        return Err(UiMountedGeometryDenial::NonFinite);
    }
    let has_surface_origin = matches!(
        input.coordinate_space,
        UiMountedCoordinateSpace::Viewport
            | UiMountedCoordinateSpace::Window
            | UiMountedCoordinateSpace::HostSurface
    );
    if has_surface_origin && (right <= 0.0 || bottom <= 0.0) {
        return Ok(UiMountedGeometryPosture::Offscreen);
    }
    Ok(UiMountedGeometryPosture::Area)
}
