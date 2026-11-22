//! Sealed test doors over this module's private members.
use super::*;

impl PreparedSettlementRegistration {
    #[doc(hidden)]
    pub fn install_for_test(
        self,
    ) -> Result<SettlementRegistrationCleanup, StoppedSettlementRegistration> {
        Self::install(self)
    }
}
