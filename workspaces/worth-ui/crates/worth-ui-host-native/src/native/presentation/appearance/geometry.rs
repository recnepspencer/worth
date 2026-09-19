use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceBackdropExtent, UiAppearanceClip,
    UiAppearanceLogicalLength, UiAppearanceVisualBounds,
};

pub(crate) const PHYSICAL_MICROS_PER_PIXEL: i64 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeAppearanceScale {
    milli: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeGeometryDenial {
    UnsupportedScale,
    CoordinateOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativePhysicalRect {
    pub(crate) left: i64,
    pub(crate) top: i64,
    pub(crate) right: i64,
    pub(crate) bottom: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativePhysicalPixelRect {
    pub(crate) left: i64,
    pub(crate) top: i64,
    pub(crate) right: i64,
    pub(crate) bottom: i64,
}

impl UiNativeAppearanceScale {
    pub(crate) fn qualified(milli: u16) -> Result<Self, UiNativeGeometryDenial> {
        if crate::native_profile::APPEARANCE_PROFILE
            .scales_milli
            .contains(&milli)
        {
            Ok(Self { milli })
        } else {
            Err(UiNativeGeometryDenial::UnsupportedScale)
        }
    }

    pub(crate) const fn milli(self) -> u16 {
        self.milli
    }

    pub(crate) fn qualified_set() -> impl ExactSizeIterator<Item = Self> {
        crate::native_profile::APPEARANCE_PROFILE
            .scales_milli
            .iter()
            .copied()
            .map(|milli| Self { milli })
    }

    pub(crate) fn scale_logical(self, value: i64) -> Result<i64, UiNativeGeometryDenial> {
        value
            .checked_mul(i64::from(self.milli))
            .ok_or(UiNativeGeometryDenial::CoordinateOverflow)
    }
}

impl UiNativePhysicalRect {
    pub(crate) fn from_allocation(
        bounds: UiAppearanceAllocationBounds,
        scale: UiNativeAppearanceScale,
    ) -> Result<Self, UiNativeGeometryDenial> {
        Self::from_edges(
            i64::from(bounds.x()),
            i64::from(bounds.y()),
            i64::from(bounds.x()) + i64::from(bounds.width()),
            i64::from(bounds.y()) + i64::from(bounds.height()),
            scale,
        )
    }

    pub(crate) fn from_visual_bounds(
        bounds: UiAppearanceVisualBounds,
        scale: UiNativeAppearanceScale,
    ) -> Result<Self, UiNativeGeometryDenial> {
        Self::from_edges(
            i64::from(bounds.x()),
            i64::from(bounds.y()),
            i64::from(bounds.x()) + i64::from(bounds.width()),
            i64::from(bounds.y()) + i64::from(bounds.height()),
            scale,
        )
    }

    pub(crate) fn from_extent(
        extent: UiAppearanceBackdropExtent,
        scale: UiNativeAppearanceScale,
    ) -> Result<Self, UiNativeGeometryDenial> {
        Self::from_edges(
            i64::from(extent.x()),
            i64::from(extent.y()),
            i64::from(extent.x()) + i64::from(extent.width()),
            i64::from(extent.y()) + i64::from(extent.height()),
            scale,
        )
    }

    pub(crate) fn from_clip(
        clip: UiAppearanceClip,
        scale: UiNativeAppearanceScale,
    ) -> Result<Self, UiNativeGeometryDenial> {
        Self::from_edges(
            i64::from(clip.x()),
            i64::from(clip.y()),
            i64::from(clip.x()) + i64::from(clip.width()),
            i64::from(clip.y()) + i64::from(clip.height()),
            scale,
        )
    }

    pub(crate) fn from_edges(
        left: i64,
        top: i64,
        right: i64,
        bottom: i64,
        scale: UiNativeAppearanceScale,
    ) -> Result<Self, UiNativeGeometryDenial> {
        let right = right
            .checked_mul(i64::from(scale.milli))
            .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
        let bottom = bottom
            .checked_mul(i64::from(scale.milli))
            .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
        Ok(Self {
            left: scale.scale_logical(left)?,
            top: scale.scale_logical(top)?,
            right,
            bottom,
        })
    }

    pub(crate) fn expand(self, amount: i64) -> Result<Self, UiNativeGeometryDenial> {
        Ok(Self {
            left: self
                .left
                .checked_sub(amount)
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?,
            top: self
                .top
                .checked_sub(amount)
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?,
            right: self
                .right
                .checked_add(amount)
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?,
            bottom: self
                .bottom
                .checked_add(amount)
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?,
        })
    }

    pub(crate) fn inset(self, amount: i64) -> Result<Self, UiNativeGeometryDenial> {
        let inset = Self {
            left: self
                .left
                .checked_add(amount)
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?,
            top: self
                .top
                .checked_add(amount)
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?,
            right: self
                .right
                .checked_sub(amount)
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?,
            bottom: self
                .bottom
                .checked_sub(amount)
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?,
        };
        if inset.left >= inset.right || inset.top >= inset.bottom {
            return Err(UiNativeGeometryDenial::CoordinateOverflow);
        }
        Ok(inset)
    }

    pub(crate) fn pixel_bounds(self) -> UiNativePhysicalPixelRect {
        UiNativePhysicalPixelRect {
            left: floor_pixel(self.left),
            top: floor_pixel(self.top),
            right: ceil_pixel(self.right),
            bottom: ceil_pixel(self.bottom),
        }
    }

    pub(crate) fn intersect(self, other: Self) -> Option<Self> {
        let result = Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        };
        (result.left < result.right && result.top < result.bottom).then_some(result)
    }
}

impl UiNativePhysicalPixelRect {
    pub(crate) fn contains(self, x: i64, y: i64) -> bool {
        self.left <= x && x < self.right && self.top <= y && y < self.bottom
    }

    pub(crate) fn intersects(self, other: Self) -> bool {
        self.left < other.right
            && other.left < self.right
            && self.top < other.bottom
            && other.top < self.bottom
    }

    pub(crate) fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    pub(crate) fn width(self) -> i64 {
        self.right - self.left
    }

    pub(crate) fn height(self) -> i64 {
        self.bottom - self.top
    }
}

pub(crate) fn physical_length(
    length: UiAppearanceLogicalLength,
    scale: UiNativeAppearanceScale,
) -> Result<i64, UiNativeGeometryDenial> {
    scale.scale_logical(i64::from(length.subpixels()))
}

pub(crate) fn physical_radii(
    radii: [u32; 4],
    scale: UiNativeAppearanceScale,
) -> Result<[i64; 4], UiNativeGeometryDenial> {
    radii
        .into_iter()
        .map(|radius| scale.scale_logical(i64::from(radius)))
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| UiNativeGeometryDenial::CoordinateOverflow)
}

pub(crate) fn pixel_center(x: i64, y: i64) -> Result<[i64; 2], UiNativeGeometryDenial> {
    [x, y]
        .into_iter()
        .map(|coordinate| {
            coordinate
                .checked_mul(PHYSICAL_MICROS_PER_PIXEL)
                .and_then(|value| value.checked_add(PHYSICAL_MICROS_PER_PIXEL / 2))
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| UiNativeGeometryDenial::CoordinateOverflow)
}

fn floor_pixel(value: i64) -> i64 {
    value.div_euclid(PHYSICAL_MICROS_PER_PIXEL)
}

fn ceil_pixel(value: i64) -> i64 {
    let quotient = value.div_euclid(PHYSICAL_MICROS_PER_PIXEL);
    let remainder = value.rem_euclid(PHYSICAL_MICROS_PER_PIXEL);
    if remainder == 0 {
        quotient
    } else {
        quotient + 1
    }
}
