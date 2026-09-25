use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
};

/// Why raw components cannot become geometry of any status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiTruthGeometryDenial {
    NonFinite,
    NegativeExtent,
}

/// The finite, non-negative rectangle every status shares as its
/// representation. It has no status of its own and never leaves this module;
/// each status wraps it and defines its own arithmetic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiLogicalRect {
    components: [u32; 4],
    coordinate_space: UiMountedCoordinateSpace,
}

impl UiLogicalRect {
    pub(super) fn new(
        components: [f32; 4],
        coordinate_space: UiMountedCoordinateSpace,
    ) -> Result<Self, UiTruthGeometryDenial> {
        if components.iter().any(|component| !component.is_finite()) {
            return Err(UiTruthGeometryDenial::NonFinite);
        }
        if components[2] < 0.0 || components[3] < 0.0 {
            return Err(UiTruthGeometryDenial::NegativeExtent);
        }
        Ok(Self {
            components: components.map(normalized_bits),
            coordinate_space,
        })
    }

    pub(super) fn from_box(bounds: UiMountedCanonicalBox) -> Self {
        Self {
            components: [bounds.x(), bounds.y(), bounds.width(), bounds.height()]
                .map(normalized_bits),
            coordinate_space: bounds.coordinate_space(),
        }
    }

    pub(super) fn components(self) -> [f32; 4] {
        self.components.map(f32::from_bits)
    }

    pub(super) const fn coordinate_space(self) -> UiMountedCoordinateSpace {
        self.coordinate_space
    }

    pub(super) fn canonical_box(self) -> UiMountedCanonicalBox {
        let [x, y, width, height] = self.components();
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: self.coordinate_space,
        })
        .expect("a finite rectangle with non-negative extent is canonical")
    }

    /// Whether a point lands in this half-open rectangle.
    pub(super) fn admits(self, point: [f32; 2]) -> bool {
        let [x, y, width, height] = self.components();
        point[0] >= x && point[1] >= y && point[0] < x + width && point[1] < y + height
    }

    /// This rectangle moved by a finite distance, in the same space.
    pub(super) fn shifted(self, by: [f32; 2]) -> Self {
        let [x, y, width, height] = self.components();
        Self::new([x + by[0], y + by[1], width, height], self.coordinate_space)
            .expect("a finite rectangle moved a finite distance stays finite")
    }

    /// Where this rectangle lands when `from` is carried onto `to`: its offset
    /// from `from` and its extent scale with `from`'s extent. It must share
    /// `from`'s space, and keeps its own. `from` has area: the maps that call
    /// this refuse a source without it.
    pub(super) fn mapped(self, from: Self, to: Self) -> Self {
        assert_eq!(
            from.coordinate_space, self.coordinate_space,
            "a mapped rectangle shares the space of the geometry that carries it"
        );
        let (
            [x, y, width, height],
            [from_x, from_y, from_width, from_height],
            [to_x, to_y, to_width, to_height],
        ) = (self.components(), from.components(), to.components());
        let (scale_x, scale_y) = (to_width / from_width, to_height / from_height);
        Self::new(
            [
                to_x + (x - from_x) * scale_x,
                to_y + (y - from_y) * scale_y,
                width * scale_x,
                height * scale_y,
            ],
            self.coordinate_space,
        )
        .expect("a validated carrying map keeps the rectangle finite")
    }
}

/// The bits a component compares by. Rects compare bitwise, so a zero
/// reached from below is stored as the zero it equals.
fn normalized_bits(component: f32) -> u32 {
    (component + 0.0).to_bits()
}
