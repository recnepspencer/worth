//! Sealed test doors over this module's private members.
use super::*;

impl PostingBindings {
    #[doc(hidden)]
    pub fn settlements_for_test(&self) -> Option<&SettlementPostingBindings> {
        Self::settlements(self)
    }
}

impl SettlementPostingBindings {
    #[doc(hidden)]
    pub fn settlements_for_test(&self) -> &OrdSet<Arc<RecordedSettlementIdentity>> {
        &self.settlements
    }
}
