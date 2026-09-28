use core::error::Error;
use core::fmt;

/// Why a value was rejected as a [`Tolerance`]. Its `Display` gives the reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidTolerance {
    reason: &'static str,
}

impl InvalidTolerance {
    fn new(reason: &'static str) -> Self {
        Self { reason }
    }
}

impl fmt::Display for InvalidTolerance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.reason)
    }
}

impl Error for InvalidTolerance {}

/// A positive length tolerance, held as a whole number of microns.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Tolerance {
    microns: u32,
}

impl Tolerance {
    /// A tolerance of the given number of microns.
    ///
    /// Fails with [`InvalidTolerance`] if `microns` is zero.
    pub fn microns(microns: u32) -> Result<Self, InvalidTolerance> {
        if microns == 0 {
            return Err(InvalidTolerance::new("tolerance must be positive"));
        }
        Ok(Self { microns })
    }

    /// The tolerance in microns.
    pub fn as_microns(&self) -> u32 {
        self.microns
    }
}
