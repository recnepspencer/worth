//! Pure ordinal for admitted external dispatch attempts.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryExternalDispatchAttemptOrdinal(pub u64);

impl WorthQueryExternalDispatchAttemptOrdinal {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn mint(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl WorthQueryExternalDispatchAttemptOrdinal {
    pub const fn value(self) -> u64 {
        self.0
    }
}
