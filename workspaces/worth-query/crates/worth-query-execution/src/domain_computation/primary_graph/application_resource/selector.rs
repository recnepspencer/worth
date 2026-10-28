//! Exact process-local selectors retain access provenance, never cleanup authority.
use super::{
    installation::InstalledResourceBinding, occurrence::OriginalResourceAccess,
    storage::ResourceRowKey,
};
use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};
use worth_query_declaration::facade::{
    application_resource::ApplicationResourceBinding, application_schema::ApplicationSchema,
};

/// A selector for one actual runtime occurrence. Cloning it grants no action,
/// disclosure, provider, or Native permission and retains no Product snapshot.
pub struct WorthQueryApplicationResourceReceipt<
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
> {
    pub(super) binding: Weak<InstalledResourceBinding<Schema, Binding>>,
    pub(super) key: ResourceRowKey,
    pub(super) access: Arc<OriginalResourceAccess>,
    marker: PhantomData<fn() -> (Schema, Binding)>,
}

impl<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>> Clone
    for WorthQueryApplicationResourceReceipt<Schema, Binding>
{
    fn clone(&self) -> Self {
        Self {
            binding: self.binding.clone(),
            key: self.key,
            access: Arc::clone(&self.access),
            marker: PhantomData,
        }
    }
}

/// Move-only public custody of an allocated runtime row. The row, rather than
/// this selector, owns pending provider, queue, Native and recovery obligations.
/// Drop marks that exact generation abandoned and performs no cancellation.
pub struct WorthQueryApplicationResource<
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
> {
    receipt: WorthQueryApplicationResourceReceipt<Schema, Binding>,
    open: bool,
}

impl<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>>
    WorthQueryApplicationResource<Schema, Binding>
{
    pub fn receipt(&self) -> WorthQueryApplicationResourceReceipt<Schema, Binding> {
        self.receipt.clone()
    }

    pub(super) fn issued(
        binding: &Arc<InstalledResourceBinding<Schema, Binding>>,
        key: ResourceRowKey,
        access: Arc<OriginalResourceAccess>,
    ) -> Self {
        Self {
            receipt: WorthQueryApplicationResourceReceipt {
                binding: Arc::downgrade(binding),
                key,
                access,
                marker: PhantomData,
            },
            open: true,
        }
    }

    pub(super) fn selector(&self) -> &WorthQueryApplicationResourceReceipt<Schema, Binding> {
        &self.receipt
    }
    pub(super) fn closed(&mut self) {
        self.open = false;
    }
}

impl<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>> Drop
    for WorthQueryApplicationResource<Schema, Binding>
{
    fn drop(&mut self) {
        if self.open
            && let Some(binding) = self.receipt.binding.upgrade()
        {
            binding.storage.abandon(self.receipt.key);
        }
    }
}
