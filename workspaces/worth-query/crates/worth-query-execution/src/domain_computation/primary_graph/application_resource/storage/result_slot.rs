use super::super::occurrence::RetainedResourceValue;
use super::super::provider::WorthQueryResourceFrameKind;
use super::*;
use std::sync::Weak;

/// Reserved before Native completion admission/commit. Its weak backing does
/// not create a storage→row→storage cycle or keep an expired runtime alive.
pub(in crate::domain_computation::primary_graph::application_resource) struct ResourceResultReservation
{
    storage: Weak<ResourceStorage>,
    index: usize,
    row: ResourceRowKey,
}

pub(in crate::domain_computation::primary_graph::application_resource) struct RetainedResourceOutput
{
    pub(in crate::domain_computation::primary_graph::application_resource) kind:
        WorthQueryResourceFrameKind,
    pub(in crate::domain_computation::primary_graph::application_resource) value:
        RetainedResourceValue,
    _slot: ResourceResultReservation,
}

impl ResourceStorage {
    pub(in crate::domain_computation::primary_graph::application_resource) fn reserve_result(
        self: &Arc<Self>,
        row: ResourceRowKey,
    ) -> Result<ResourceResultReservation, ResourceStorageDenial> {
        let mut free = lock(&self.free);
        let index = free
            .result
            .ok_or(ResourceStorageDenial::RetainedResultCapacity)?;
        let mut result = lock(&self.results[index]);
        free.result = result.next_free.take();
        result.owner = Some(row);
        Ok(ResourceResultReservation {
            storage: Arc::downgrade(self),
            index,
            row,
        })
    }
}

impl ResourceResultReservation {
    pub(in crate::domain_computation::primary_graph::application_resource) fn retain(
        self,
        kind: WorthQueryResourceFrameKind,
        value: RetainedResourceValue,
    ) -> RetainedResourceOutput {
        RetainedResourceOutput {
            kind,
            value,
            _slot: self,
        }
    }
}

impl Drop for ResourceResultReservation {
    fn drop(&mut self) {
        let Some(storage) = self.storage.upgrade() else {
            return;
        };
        let mut free = lock(&storage.free);
        let mut slot = lock(&storage.results[self.index]);
        if slot.owner != Some(self.row) {
            return;
        }
        slot.owner = None;
        slot.next_free = free.result;
        free.result = Some(self.index);
    }
}
