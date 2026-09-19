#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Bounds(pub(super) [f64; 4]);

impl Bounds {
    pub(super) fn admit(raw: [f64; 4]) -> Result<Option<Self>, ()> {
        if raw.iter().any(|value| !value.is_finite()) || raw[0] > raw[2] || raw[1] > raw[3] {
            return Err(());
        }
        Ok((raw[0] < raw[2] && raw[1] < raw[3]).then_some(Self(raw)))
    }

    pub(super) fn union(self, other: Self) -> Self {
        Self([
            self.0[0].min(other.0[0]),
            self.0[1].min(other.0[1]),
            self.0[2].max(other.0[2]),
            self.0[3].max(other.0[3]),
        ])
    }

    pub(super) fn contains(self, point: [f64; 2]) -> bool {
        self.0[0] <= point[0]
            && point[0] < self.0[2]
            && self.0[1] <= point[1]
            && point[1] < self.0[3]
    }

    pub(super) fn intersects(self, other: Self) -> bool {
        self.0[0] < other.0[2]
            && other.0[0] < self.0[2]
            && self.0[1] < other.0[3]
            && other.0[1] < self.0[3]
    }
}
