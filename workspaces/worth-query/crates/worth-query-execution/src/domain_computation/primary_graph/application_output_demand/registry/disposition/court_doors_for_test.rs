//! Sealed test doors over this module's private members.
use super::*;

impl DemandRecord {
    #[doc(hidden)]
    pub fn terminal_for_test(&self) -> bool {
        Self::terminal(self)
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn idle_disposition_for_test(&self, awaited_by_stale_owner: bool) -> IdleRowDisposition {
        Self::idle_disposition(self, awaited_by_stale_owner)
    }
}
