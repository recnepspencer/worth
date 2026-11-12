/// Stable owner-section tags. Their numbers do not prescribe recovery order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DurableOwnerFamily {
    Relational,
    Signal,
    Bridge,
    World,
    Query,
    Dispatch,
    Correction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnsupportedDurableOwnerTag {
    pub observed: u16,
}

impl DurableOwnerFamily {
    pub const fn tag(self) -> u16 {
        match self {
            Self::Relational => 1,
            Self::Signal => 2,
            Self::Bridge => 3,
            Self::World => 4,
            Self::Query => 5,
            Self::Dispatch => 6,
            Self::Correction => 7,
        }
    }

    pub const fn from_tag(observed: u16) -> Result<Self, UnsupportedDurableOwnerTag> {
        match observed {
            1 => Ok(Self::Relational),
            2 => Ok(Self::Signal),
            3 => Ok(Self::Bridge),
            4 => Ok(Self::World),
            5 => Ok(Self::Query),
            6 => Ok(Self::Dispatch),
            7 => Ok(Self::Correction),
            _ => Err(UnsupportedDurableOwnerTag { observed }),
        }
    }
}
