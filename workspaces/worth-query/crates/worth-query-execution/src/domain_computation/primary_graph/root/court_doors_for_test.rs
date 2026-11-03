//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn layout_for_test(&self) -> &Arc<WorthQueryPrimaryGraphLayout> {
        &self.layout
    }
}

impl WorthQueryPrimaryGraph {
    #[doc(hidden)]
    pub fn integration_handle_for_test(&self) -> WorthQueryPrimaryGraphIntegrationHandle {
        Self::integration_handle(self)
    }
}

impl WorthQueryPrimaryGraph {
    #[doc(hidden)]
    pub fn layout_for_test(&self) -> &WorthQueryPrimaryGraphLayout {
        Self::layout(self)
    }
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn primary_index_ids_for_test(&self) -> &Arc<[worth_relational::facade::indexes::DerivedIndexId]> {
        &self.primary_index_ids
    }
}
