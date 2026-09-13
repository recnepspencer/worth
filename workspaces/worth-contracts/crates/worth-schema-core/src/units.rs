#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Unit {
    Millimeters,
    Meters,
    Degrees,
    Radians,
}

impl Unit {
    pub fn millimeters() -> Self {
        Self::Millimeters
    }

    pub fn meters() -> Self {
        Self::Meters
    }

    pub fn degrees() -> Self {
        Self::Degrees
    }

    pub fn radians() -> Self {
        Self::Radians
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Millimeters => "mm",
            Self::Meters => "m",
            Self::Degrees => "deg",
            Self::Radians => "rad",
        }
    }
}
