/// Historical incarnation data. Possession is not a fenced capability.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HomeIncarnation(std::num::NonZeroU64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeIncarnationExhausted;

impl HomeIncarnation {
    pub const fn initial() -> Self {
        Self(std::num::NonZeroU64::MIN)
    }

    pub const fn from_recorded(value: u64) -> Option<Self> {
        match std::num::NonZeroU64::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }

    /// Calculate a successor value; only the home owner may publish it and
    /// issue authority for that incarnation.
    pub fn successor(self) -> Result<Self, HomeIncarnationExhausted> {
        self.get()
            .checked_add(1)
            .and_then(Self::from_recorded)
            .ok_or(HomeIncarnationExhausted)
    }
}

impl super::identity_seal::Sealed for HomeIncarnation {}
impl super::DurableIdentity for HomeIncarnation {}
