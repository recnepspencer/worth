//! Sealed test doors over this module's private members.
use super::*;

impl CheckpointDecodeDenial {
    #[doc(hidden)]
    pub fn from_fact_for_test(cause: FactDecodeDenial) -> Self {
        Self::from_fact(cause)
    }
}

impl CheckpointDecodeDenial {
    #[doc(hidden)]
    pub fn into_installation_denial_for_test(self) -> Denial {
        Self::into_installation_denial(self)
    }
}
