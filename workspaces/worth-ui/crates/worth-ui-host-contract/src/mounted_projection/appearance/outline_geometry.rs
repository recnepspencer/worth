#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceOutlineGeometry {
    allocation: super::UiAppearanceAllocationBounds,
    visual_bounds: super::UiAppearanceVisualBounds,
    width: super::UiAppearanceLogicalLength,
    offset: super::UiAppearanceLogicalLength,
    anti_alias_fringe: super::UiAppearanceLogicalLength,
    radii: super::UiAppearanceNormalizedLogicalRadii,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceOutlineGeometryDenial {
    RadiiAllocationMismatch,
    GeometryOverflow,
}

impl UiAppearanceOutlineGeometry {
    pub fn admit(
        allocation: super::UiAppearanceAllocationBounds,
        surface_radii: super::UiAppearanceNormalizedLogicalRadii,
        width: super::UiAppearanceLogicalLength,
        offset: super::UiAppearanceLogicalLength,
        anti_alias_fringe: super::UiAppearanceLogicalLength,
    ) -> Result<Self, UiAppearanceOutlineGeometryDenial> {
        if !surface_radii.matches_allocation(allocation) {
            return Err(UiAppearanceOutlineGeometryDenial::RadiiAllocationMismatch);
        }
        let expansion = width
            .subpixels()
            .checked_add(offset.subpixels())
            .and_then(|value| value.checked_add(anti_alias_fringe.subpixels()))
            .ok_or(UiAppearanceOutlineGeometryDenial::GeometryOverflow)?;
        let visual_bounds = super::UiAppearanceVisualBounds(
            allocation
                .0
                .expanded(expansion)
                .map_err(|_| UiAppearanceOutlineGeometryDenial::GeometryOverflow)?,
        );
        let radii = surface_radii
            .with_outline_offset(offset)
            .map_err(|_| UiAppearanceOutlineGeometryDenial::GeometryOverflow)?;
        Ok(Self {
            allocation,
            visual_bounds,
            width,
            offset,
            anti_alias_fringe,
            radii,
        })
    }

    pub const fn allocation(self) -> super::UiAppearanceAllocationBounds {
        self.allocation
    }
    pub const fn visual_bounds(self) -> super::UiAppearanceVisualBounds {
        self.visual_bounds
    }
    pub const fn width(self) -> super::UiAppearanceLogicalLength {
        self.width
    }
    pub const fn offset(self) -> super::UiAppearanceLogicalLength {
        self.offset
    }
    pub const fn anti_alias_fringe(self) -> super::UiAppearanceLogicalLength {
        self.anti_alias_fringe
    }
    pub const fn radii(self) -> super::UiAppearanceNormalizedLogicalRadii {
        self.radii
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn length(value: i32) -> super::super::UiAppearanceLogicalLength {
        super::super::UiAppearanceLogicalLength::new(value).unwrap()
    }

    #[test]
    fn outline_geometry_derives_exact_visual_expansion_and_radii() {
        let allocation =
            super::super::UiAppearanceAllocationBounds::new(10_000, 20_000, 30_000, 40_000)
                .unwrap();
        let surface_radii = super::super::UiAppearanceNormalizedLogicalRadii::normalize(
            allocation,
            [length(4_000); 4],
        );
        let geometry = UiAppearanceOutlineGeometry::admit(
            allocation,
            surface_radii,
            length(2_000),
            length(1_000),
            length(500),
        )
        .unwrap();
        assert_eq!(
            [geometry.visual_bounds().x(), geometry.visual_bounds().y()],
            [6_500, 16_500]
        );
        assert_eq!(
            [
                geometry.visual_bounds().width(),
                geometry.visual_bounds().height(),
            ],
            [37_000, 47_000]
        );
        assert_eq!(geometry.radii().corners(), [5_000; 4]);
        let offset_basis =
            super::super::UiAppearanceAllocationBounds::new(9_000, 19_000, 32_000, 42_000).unwrap();
        assert!(!geometry.radii().matches_allocation(allocation));
        assert!(geometry.radii().matches_allocation(offset_basis));
    }

    #[test]
    fn outline_expansion_overflow_denies_before_mechanic_completion() {
        let allocation =
            super::super::UiAppearanceAllocationBounds::new(i32::MIN, 0, 1, 1).unwrap();
        let radii = super::super::UiAppearanceNormalizedLogicalRadii::normalize(
            allocation,
            [super::super::UiAppearanceLogicalLength::ZERO; 4],
        );
        assert_eq!(
            UiAppearanceOutlineGeometry::admit(
                allocation,
                radii,
                length(1),
                super::super::UiAppearanceLogicalLength::ZERO,
                super::super::UiAppearanceLogicalLength::ZERO,
            ),
            Err(UiAppearanceOutlineGeometryDenial::GeometryOverflow)
        );
    }

    #[test]
    fn outline_geometry_denies_radii_normalized_for_another_allocation() {
        let large = super::super::UiAppearanceAllocationBounds::new(0, 0, 100, 100).unwrap();
        let small = super::super::UiAppearanceAllocationBounds::new(0, 0, 10, 10).unwrap();
        let radii =
            super::super::UiAppearanceNormalizedLogicalRadii::normalize(large, [length(40); 4]);
        assert_eq!(
            UiAppearanceOutlineGeometry::admit(small, radii, length(1), length(1), length(1),),
            Err(UiAppearanceOutlineGeometryDenial::RadiiAllocationMismatch)
        );
    }
}
