#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiSpatialRect {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSpatialProjectionDenial {
    UnsupportedCoordinateSpace,
    InvalidGeometry,
}

pub(crate) fn project_clipped_region(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    clip: worth_ui_host_contract::UiMountedCanonicalBox,
    transform: worth_ui_host_contract::UiHostCoordinateTransform,
) -> Result<Option<UiSpatialRect>, UiSpatialProjectionDenial> {
    use worth_ui_host_contract::{UiMountedCoordinateSpace, UiMountedGeometryPosture};

    if bounds.coordinate_space() != UiMountedCoordinateSpace::Viewport
        || clip.coordinate_space() != UiMountedCoordinateSpace::Viewport
    {
        return Err(UiSpatialProjectionDenial::UnsupportedCoordinateSpace);
    }
    // A fully clipped Scroll descendant is a valid host row with no visible
    // or hit-test pixels. Its exact mechanic is still matched by the caller;
    // only the spatial index entry is omitted.
    if bounds.posture() != UiMountedGeometryPosture::Area {
        return Err(UiSpatialProjectionDenial::InvalidGeometry);
    }
    if clip.posture() == UiMountedGeometryPosture::Empty {
        return Ok(None);
    }
    if clip.posture() != UiMountedGeometryPosture::Area {
        return Err(UiSpatialProjectionDenial::InvalidGeometry);
    }
    let translation = transform.translation();
    let logical = transform.viewport_logical_dimensions();
    let viewport = [
        translation[0],
        translation[1],
        translation[0] + logical[0],
        translation[1] + logical[1],
    ];
    let Some(clipped_to_viewport) = intersect(box_edges(clip), viewport) else {
        return Ok(None);
    };
    let logical_region = intersect(box_edges(bounds), clipped_to_viewport);
    let Some(region) = logical_region else {
        return Ok(None);
    };
    project_logical_rect(region, transform).map(Some)
}

fn project_logical_rect(
    region: [f32; 4],
    transform: worth_ui_host_contract::UiHostCoordinateTransform,
) -> Result<UiSpatialRect, UiSpatialProjectionDenial> {
    let translation = transform.translation();
    let scale = transform.scale();
    let dimensions = transform.client_physical_dimensions();
    let left = project_edge(region[0] - translation[0], scale[0], transform.rounding())?;
    let right = project_edge(region[2] - translation[0], scale[0], transform.rounding())?;
    let (top, bottom) = match transform.orientation() {
        worth_ui_host_contract::UiHostCoordinateOrientation::TopLeftOrigin => (
            project_edge(region[1] - translation[1], scale[1], transform.rounding())?,
            project_edge(region[3] - translation[1], scale[1], transform.rounding())?,
        ),
        worth_ui_host_contract::UiHostCoordinateOrientation::BottomLeftOrigin => (
            dimensions[1].saturating_sub(project_edge(
                region[3] - translation[1],
                scale[1],
                transform.rounding(),
            )?),
            dimensions[1].saturating_sub(project_edge(
                region[1] - translation[1],
                scale[1],
                transform.rounding(),
            )?),
        ),
    };
    UiSpatialRect::new(
        left.min(dimensions[0]),
        top.min(dimensions[1]),
        right.min(dimensions[0]),
        bottom.min(dimensions[1]),
    )
}

fn project_edge(
    logical: f32,
    scale: f32,
    rounding: worth_ui_host_contract::UiHostCoordinateRounding,
) -> Result<u32, UiSpatialProjectionDenial> {
    let physical = logical * scale;
    if !physical.is_finite() || physical < 0.0 || physical > u32::MAX as f32 {
        return Err(UiSpatialProjectionDenial::InvalidGeometry);
    }
    let rounded = match rounding {
        worth_ui_host_contract::UiHostCoordinateRounding::PixelCenterNearest => physical.round(),
        worth_ui_host_contract::UiHostCoordinateRounding::FloorEdges => physical.floor(),
    };
    Ok(rounded as u32)
}

fn box_edges(bounds: worth_ui_host_contract::UiMountedCanonicalBox) -> [f32; 4] {
    [
        bounds.x(),
        bounds.y(),
        bounds.x() + bounds.width(),
        bounds.y() + bounds.height(),
    ]
}

fn intersect(left: [f32; 4], right: [f32; 4]) -> Option<[f32; 4]> {
    let intersection = [
        left[0].max(right[0]),
        left[1].max(right[1]),
        left[2].min(right[2]),
        left[3].min(right[3]),
    ];
    (intersection[0] < intersection[2] && intersection[1] < intersection[3]).then_some(intersection)
}

impl UiSpatialRect {
    fn new(
        left: u32,
        top: u32,
        right: u32,
        bottom: u32,
    ) -> Result<Self, UiSpatialProjectionDenial> {
        if left >= right || top >= bottom {
            return Err(UiSpatialProjectionDenial::InvalidGeometry);
        }
        Ok(Self {
            left,
            top,
            right,
            bottom,
        })
    }

    pub(crate) const fn left(self) -> u32 {
        self.left
    }

    pub(crate) const fn right(self) -> u32 {
        self.right
    }

    pub(crate) const fn contains(self, point: worth_ui_inspection::UiClientPhysicalPixel) -> bool {
        point.x() >= self.left
            && point.x() < self.right
            && point.y() >= self.top
            && point.y() < self.bottom
    }

    pub(crate) fn inspection_rect(self) -> worth_ui_inspection::UiClientPhysicalRect {
        worth_ui_inspection::UiClientPhysicalRect::new(self.left, self.top, self.right, self.bottom)
            .expect("validated nonempty spatial rectangles project to inspection rectangles")
    }

    pub(crate) const fn intersects(
        self,
        region: worth_ui_inspection::UiClientPhysicalRect,
    ) -> bool {
        self.left < region.right()
            && self.right > region.left()
            && self.top < region.bottom()
            && self.bottom > region.top()
    }
}
