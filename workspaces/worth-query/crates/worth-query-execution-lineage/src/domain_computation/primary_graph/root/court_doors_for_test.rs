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
    pub fn primary_index_ids_for_test(
        &self,
    ) -> &Arc<[worth_relational::facade::indexes::DerivedIndexId]> {
        &self.primary_index_ids
    }
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn aggregate_projections_for_test(
        &self,
    ) -> &Arc<Mutex<super::super::aggregate_projection::WorthQueryAggregateProjections>> {
        &self.aggregate_projections
    }
}

impl WorthQueryPrimaryGraph {
    #[doc(hidden)]
    pub fn managed_derived_views_for_test(&self) -> crate::ports::ManagedDerivedViewCourtDoor {
        crate::ports::ManagedDerivedViewCourtDoor(Self::managed_derived_views(self))
    }
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn source_owner_for_test(&self) -> &home_retirement::PrimaryGraphSourceCustody {
        &self.source_owner
    }
}
