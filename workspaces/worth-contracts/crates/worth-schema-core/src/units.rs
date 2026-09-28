/// A unit of measure for lengths and angles.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Unit {
    /// Millimeters, a length.
    Millimeters,
    /// Meters, a length.
    Meters,
    /// Degrees, an angle.
    Degrees,
    /// Radians, an angle.
    Radians,
}

impl Unit {
    /// The millimeter unit.
    pub fn millimeters() -> Self {
        Self::Millimeters
    }

    /// The meter unit.
    pub fn meters() -> Self {
        Self::Meters
    }

    /// The degree unit.
    pub fn degrees() -> Self {
        Self::Degrees
    }

    /// The radian unit.
    pub fn radians() -> Self {
        Self::Radians
    }

    /// The short symbol for the unit: `mm`, `m`, `deg` or `rad`.
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Millimeters => "mm",
            Self::Meters => "m",
            Self::Degrees => "deg",
            Self::Radians => "rad",
        }
    }
}
