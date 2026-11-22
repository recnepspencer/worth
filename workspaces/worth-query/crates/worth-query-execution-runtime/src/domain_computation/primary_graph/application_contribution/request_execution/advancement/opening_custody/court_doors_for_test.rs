//! Sealed test doors over this module's private members.
use super::*;

impl OpeningCustody {
    #[doc(hidden)]
    pub fn enter_for_test() -> Result<Self, WorthQueryAdvancementDenial> {
        Self::enter()
    }
}
