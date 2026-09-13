use super::{NormalizedPoint, UiMountedAppearanceColor};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedSurfaceFill {
    Solid(UiMountedAppearanceColor),
    LinearGradient(UiMountedLinearGradient),
}

impl From<UiMountedAppearanceColor> for UiMountedSurfaceFill {
    fn from(value: UiMountedAppearanceColor) -> Self {
        Self::Solid(value)
    }
}

impl UiMountedSurfaceFill {
    pub const fn colors(self) -> [UiMountedAppearanceColor; 2] {
        match self {
            Self::Solid(color) => [color; 2],
            Self::LinearGradient(gradient) => gradient.colors(),
        }
    }

    pub fn sample(self, allocation_edges: [f64; 4], point: [f64; 2]) -> UiMountedAppearanceColor {
        let Self::LinearGradient(gradient) = self else {
            return self.colors()[0];
        };
        let map = |p: NormalizedPoint| {
            let [x, y] = p.coordinates().map(|v| f64::from(v) / 10_000.0);
            [
                allocation_edges[0] + x * (allocation_edges[2] - allocation_edges[0]),
                allocation_edges[1] + y * (allocation_edges[3] - allocation_edges[1]),
            ]
        };
        let start = map(gradient.start());
        let end = map(gradient.end());
        let d = [end[0] - start[0], end[1] - start[1]];
        let length_squared = d[0] * d[0] + d[1] * d[1];
        let fraction = if length_squared == 0.0 {
            0.0
        } else {
            (((point[0] - start[0]) * d[0] + (point[1] - start[1]) * d[1]) / length_squared)
                .clamp(0.0, 1.0)
        };
        super::compositing::interpolate_linear(
            gradient.colors(),
            (fraction * 65_536.0).round() as u32,
        )
    }
}

/// Clamped premultiplied-linear-sRGB interpolation, in allocation coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedLinearGradient {
    start: NormalizedPoint,
    end: NormalizedPoint,
    colors: [UiMountedAppearanceColor; 2],
}

impl UiMountedLinearGradient {
    pub fn new(
        start: NormalizedPoint,
        end: NormalizedPoint,
        colors: [UiMountedAppearanceColor; 2],
    ) -> Option<Self> {
        (start != end).then_some(Self { start, end, colors })
    }
    pub const fn start(self) -> NormalizedPoint {
        self.start
    }
    pub const fn end(self) -> NormalizedPoint {
        self.end
    }
    pub const fn colors(self) -> [UiMountedAppearanceColor; 2] {
        self.colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_interpolation_uses_physical_axis_and_premultiplied_linear_color() {
        let point = |x, y| NormalizedPoint::new(x, y).unwrap();
        let color = UiMountedAppearanceColor::from_straight_srgba;
        let make = |colors: [[u8; 4]; 2]| {
            UiMountedSurfaceFill::LinearGradient(
                UiMountedLinearGradient::new(point(0, 0), point(10_000, 10_000), colors.map(color))
                    .unwrap(),
            )
        };
        let ramp = make([[0, 0, 0, 255], [255, 255, 255, 255]]);
        let bounds = [20.0, 10.0, 220.0, 110.0];
        // Independent oracle: physical direction (200,100). At (20,110),
        // t=10000/50000=.2, whose linear-light sRGB encoding is 124.
        assert_eq!(
            ramp.sample(bounds, [20.0, 110.0]).straight_srgba(),
            [124, 124, 124, 255]
        );
        assert_eq!(
            ramp.sample(bounds, [120.0, 60.0]).straight_srgba(),
            [188, 188, 188, 255]
        );
        assert_eq!(
            ramp.sample(bounds, [-100.0, -100.0]).straight_srgba(),
            [0, 0, 0, 255]
        );
        assert_eq!(
            ramp.sample(bounds, [400.0, 400.0]).straight_srgba(),
            [255; 4]
        );
        let fade = make([[255, 0, 0, 255], [0, 0, 255, 0]]);
        assert_eq!(
            fade.sample(bounds, [120.0, 60.0]).straight_srgba(),
            [255, 0, 0, 128]
        );
        assert!(
            UiMountedLinearGradient::new(point(0, 0), point(0, 0), [color([0; 4]); 2]).is_none()
        );
    }
}
