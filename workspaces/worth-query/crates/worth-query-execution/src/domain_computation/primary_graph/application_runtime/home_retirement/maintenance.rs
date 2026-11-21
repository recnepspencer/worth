//! Home maintenance custody access during primary-graph retirement.

use std::sync::Arc;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;

use super::RetiringPrimaryGraphRuntime;
use crate::domain_computation::execution_runtime::product_world::{
    HomeMaintenanceAdapter, WorthQueryHomeMaintenance,
};

impl<Schema: ApplicationSchema> RetiringPrimaryGraphRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn home_maintenance_mut(
        &mut self,
    ) -> Option<&mut Arc<dyn WorthQueryHomeMaintenance>> {
        self.runtime
            .inner
            .runtime
            .product_runtime
            .home_maintenance_slot_mut()
            .as_mut()
            .and_then(Arc::get_mut)
            .and_then(|adapter| {
                adapter
                    .as_any_mut()
                    .downcast_mut::<HomeMaintenanceAdapter>()
            })
            .map(HomeMaintenanceAdapter::inner_mut)
    }

    pub(in crate::domain_computation::primary_graph) fn take_home_maintenance(
        &mut self,
    ) -> Option<Arc<dyn WorthQueryHomeMaintenance>> {
        let taken = self
            .runtime
            .inner
            .runtime
            .product_runtime
            .home_maintenance_slot_mut()
            .take()?;
        let inner = taken
            .as_any()
            .downcast_ref::<HomeMaintenanceAdapter>()
            .map(|adapter| Arc::clone(&adapter.0));
        drop(taken);
        inner
    }
}
