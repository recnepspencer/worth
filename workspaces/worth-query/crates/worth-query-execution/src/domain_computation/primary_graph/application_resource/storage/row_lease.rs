//! A row action releases its lock before arbitrary provider code and restores
//! the same occurrence even when an original request or callback unwinds.
use super::*;

pub(in crate::domain_computation::primary_graph::application_resource) struct ResourceRowLease {
    storage: Arc<ResourceStorage>,
    key: ResourceRowKey,
    occurrence: Option<ResourceOccurrence>,
}

impl ResourceRowLease {
    pub(super) fn new(
        storage: Arc<ResourceStorage>,
        key: ResourceRowKey,
        occurrence: ResourceOccurrence,
    ) -> Self {
        Self {
            storage,
            key,
            occurrence: Some(occurrence),
        }
    }

    pub(in crate::domain_computation::primary_graph::application_resource) fn key(
        &self,
    ) -> ResourceRowKey {
        self.key
    }
    pub(in crate::domain_computation::primary_graph::application_resource) fn occurrence(
        &self,
    ) -> &ResourceOccurrence {
        self.occurrence
            .as_ref()
            .expect("live row action owns its payload")
    }
    pub(in crate::domain_computation::primary_graph::application_resource) fn occurrence_mut(
        &mut self,
    ) -> &mut ResourceOccurrence {
        self.occurrence
            .as_mut()
            .expect("live row action owns its payload")
    }

    /// Close calls this only after actual cleanup/recovery handoff. Merely
    /// dropping a public selector or refusing an action never reaches it.
    pub(in crate::domain_computation::primary_graph::application_resource) fn finish_native_slot(
        &mut self,
    ) -> Result<(), ResourceStorageDenial> {
        use super::super::occurrence::ResourceLifecycle;
        let occurrence = self.occurrence_mut();
        let native_resolved = match &occurrence.lifecycle {
            ResourceLifecycle::Released(_) => true,
            ResourceLifecycle::Reserved { .. } => occurrence.attempt.is_none(),
            ResourceLifecycle::Live { .. } => false,
        };
        if !occurrence.provider_closed || occurrence.candidate.is_some() || !native_resolved {
            return Err(ResourceStorageDenial::CleanupPending);
        }
        let index = occurrence.native_slot.take();
        if let Some(index) = index {
            self.storage.return_native(index);
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_resource) fn recycle(
        mut self,
    ) -> Result<super::super::closed::WorthQueryClosedResource, (Self, ResourceStorageDenial)> {
        use super::super::occurrence::ResourceLifecycle;
        let occurrence = self.occurrence();
        if occurrence.recovery.is_some() {
            return Err((self, ResourceStorageDenial::RecoveryHandoffRequired));
        }
        if occurrence.native_slot.is_some()
            || !occurrence.provider_closed
            || occurrence.candidate.is_some()
            || matches!(occurrence.lifecycle, ResourceLifecycle::Live { .. })
        {
            return Err((self, ResourceStorageDenial::CleanupPending));
        }
        if occurrence.last_action.is_none() {
            return Err((self, ResourceStorageDenial::CleanupPending));
        }
        let mut occurrence = self
            .occurrence
            .take()
            .expect("closed action retains actual row");
        let mut free = lock(&self.storage.free);
        let mut slot = lock(&self.storage.rows[self.key.index]);
        assert!(
            slot.allocated && slot.generation == self.key.generation && slot.occurrence.is_none()
        );
        slot.allocated = false;
        slot.next_free = free.row;
        free.row = Some(self.key.index);
        drop(slot);
        drop(free);
        // A retained output releases its result slot through the same free
        // index owner, so do not run its Drop while holding that owner's lock.
        let (native, clock_only) = match occurrence.lifecycle {
            ResourceLifecycle::Released(native) => (Some(native), None),
            ResourceLifecycle::Reserved { clock } => (None, clock),
            ResourceLifecycle::Live { .. } => unreachable!("live owner cannot be recycled"),
        };
        let action = occurrence
            .last_action
            .take()
            .expect("actual ResourceRead terminal precedes recycling");
        // The remaining fields, including result-slot and original input
        // funding, release after the shared free-list locks are gone.
        drop(occurrence.result.take());
        Ok(super::super::closed::WorthQueryClosedResource {
            native,
            clock_only,
            action,
            detail: None,
        })
    }
}

impl Drop for ResourceRowLease {
    fn drop(&mut self) {
        let Some(occurrence) = self.occurrence.take() else {
            return;
        };
        let mut slot = lock(&self.storage.rows[self.key.index]);
        assert!(
            slot.allocated && slot.generation == self.key.generation && slot.occurrence.is_none()
        );
        // Never overwrite the abandonment flag: public Drop can set it while
        // this payload is outside the row during a provider callback.
        slot.occurrence = Some(occurrence);
    }
}
