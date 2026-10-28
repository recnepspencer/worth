//! One typed resource batch is installed last on the existing conditional graph.
mod clock;
mod pending;
mod pool;
use crate::equality_type_id::EqualityTypeId as TypeId;
pub(super) use clock::ResourceClock;
pub(in crate::domain_computation::primary_graph) use pending::PendingResourceRegistry;
pub(super) use pool::InstalledResourceBinding;
use std::{
    any::Any,
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
    sync::Arc,
};
use worth_query_declaration::facade::{
    application_resource::ApplicationResourceBinding, application_schema::ApplicationSchema,
};

pub(in crate::domain_computation::primary_graph) struct InstalledResourceRegistry<Schema> {
    // Pending installation supplies canonical resource-name order.
    entries: Vec<InstalledResourceEntry>,
    by_binding: HashMap<TypeId, usize>,
    marker: PhantomData<fn() -> Schema>,
}

struct InstalledResourceEntry {
    binding: Arc<dyn Any + Send + Sync>,
    storage: Arc<super::storage::ResourceStorage>,
}

impl<Schema> Default for InstalledResourceRegistry<Schema> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            by_binding: HashMap::new(),
            marker: PhantomData,
        }
    }
}

impl<Schema> InstalledResourceRegistry<Schema> {
    pub(in crate::domain_computation::primary_graph) fn cleanup_retained(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.storage.allocated_rows_retained())
    }
    pub(in crate::domain_computation::primary_graph) fn clear(&mut self) {
        self.entries.clear();
        self.by_binding.clear();
    }
}

impl<Schema: ApplicationSchema> InstalledResourceRegistry<Schema> {
    pub(super) fn binding<Binding: ApplicationResourceBinding<Schema>>(
        &self,
    ) -> Option<Arc<InstalledResourceBinding<Schema, Binding>>> {
        self.entries
            .get(*self.by_binding.get(&TypeId::of::<Binding>())?)?
            .binding
            .clone()
            .downcast()
            .ok()
    }
}

fn denial(detail: impl Into<String>) -> super::super::WorthQueryPrimaryGraphInstallationDenial {
    super::super::WorthQueryPrimaryGraphInstallationDenial::new(
        super::super::WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
        detail,
    )
}
