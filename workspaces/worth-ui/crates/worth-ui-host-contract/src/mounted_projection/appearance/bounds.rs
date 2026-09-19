#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceLogicalRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceAllocationBounds(pub(crate) UiAppearanceLogicalRect);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceVisualBounds(pub(crate) UiAppearanceLogicalRect);

/// Exact presented backdrop extent, independent of repaint damage and clipping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceBackdropExtent(pub(crate) UiAppearanceLogicalRect);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceDamageRegion(pub(crate) UiAppearanceLogicalRect);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceClip(pub(crate) UiAppearanceLogicalRect);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceEmptyRegion;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceGeometryOverflow;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceDamageAttribution {
    Surface,
    Outline,
    TextForeground,
    PortalSurface,
    Backdrop,
}

impl UiAppearanceLogicalRect {
    const fn new(x: i32, y: i32, width: u32, height: u32) -> Result<Self, UiAppearanceEmptyRegion> {
        if width == 0 || height == 0 {
            Err(UiAppearanceEmptyRegion)
        } else {
            Ok(Self {
                x,
                y,
                width,
                height,
            })
        }
    }

    pub(crate) fn expanded(self, amount: u32) -> Result<Self, UiAppearanceGeometryOverflow> {
        let amount_i64 = i64::from(amount);
        let x = i64::from(self.x) - amount_i64;
        let y = i64::from(self.y) - amount_i64;
        let width = u64::from(self.width) + u64::from(amount) * 2;
        let height = u64::from(self.height) + u64::from(amount) * 2;
        Ok(Self {
            x: i32::try_from(x).map_err(|_| UiAppearanceGeometryOverflow)?,
            y: i32::try_from(y).map_err(|_| UiAppearanceGeometryOverflow)?,
            width: u32::try_from(width).map_err(|_| UiAppearanceGeometryOverflow)?,
            height: u32::try_from(height).map_err(|_| UiAppearanceGeometryOverflow)?,
        })
    }
}

macro_rules! logical_rect_contract {
    ($name:ident) => {
        impl $name {
            pub const fn new(
                x: i32,
                y: i32,
                width: u32,
                height: u32,
            ) -> Result<Self, UiAppearanceEmptyRegion> {
                match UiAppearanceLogicalRect::new(x, y, width, height) {
                    Ok(rect) => Ok(Self(rect)),
                    Err(denial) => Err(denial),
                }
            }

            pub const fn x(self) -> i32 {
                self.0.x
            }
            pub const fn y(self) -> i32 {
                self.0.y
            }
            pub const fn width(self) -> u32 {
                self.0.width
            }
            pub const fn height(self) -> u32 {
                self.0.height
            }
        }
    };
}

logical_rect_contract!(UiAppearanceAllocationBounds);
logical_rect_contract!(UiAppearanceBackdropExtent);
logical_rect_contract!(UiAppearanceDamageRegion);
logical_rect_contract!(UiAppearanceClip);

impl UiAppearanceVisualBounds {
    pub const fn from_surface_allocation(allocation: UiAppearanceAllocationBounds) -> Self {
        Self(allocation.0)
    }

    pub const fn x(self) -> i32 {
        self.0.x
    }
    pub const fn y(self) -> i32 {
        self.0.y
    }
    pub const fn width(self) -> u32 {
        self.0.width
    }
    pub const fn height(self) -> u32 {
        self.0.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_regions_deny_for_each_caller_owned_region() {
        assert_eq!(
            UiAppearanceAllocationBounds::new(0, 0, 0, 1),
            Err(UiAppearanceEmptyRegion)
        );
        assert_eq!(
            UiAppearanceBackdropExtent::new(0, 0, 0, 1),
            Err(UiAppearanceEmptyRegion)
        );
        assert_eq!(
            UiAppearanceDamageRegion::new(0, 0, 1, 0),
            Err(UiAppearanceEmptyRegion)
        );
        assert_eq!(
            UiAppearanceClip::new(0, 0, 0, 0),
            Err(UiAppearanceEmptyRegion)
        );
    }
}
