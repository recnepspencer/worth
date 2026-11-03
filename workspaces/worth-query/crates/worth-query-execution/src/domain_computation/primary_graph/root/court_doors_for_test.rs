//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn layout_for_test(&self) -> &Arc<WorthQueryPrimaryGraphLayout> {
        &self.layout
    }
}
