use super::{CollisionExtents, PersistentHashMap, PersistentHashMapStorage};
use crate::data::retained_storage::{
    RetainedStorageBacking, SignalConditionalRetentionReservation,
};
use std::sync::Arc;

impl<K: Clone + Eq + std::hash::Hash, V: Clone> PersistentHashMap<K, V> {
    /// Construct an empty overlay whose subsequent clones share their HAMT
    /// storage instead of copying an exclusive table.
    pub(crate) fn new_persistent_overlay() -> Self {
        let mut overlay = Self::new();
        drop(overlay.fork_persistent());
        overlay
    }

    pub(crate) fn fork_persistent(&mut self) -> Self {
        self.fork_with_resources(None)
    }

    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        self.fork_with_resources(Some(resources))
    }

    fn fork_with_resources(
        &mut self,
        resources: Option<&mut SignalConditionalRetentionReservation>,
    ) -> Self {
        let charge = self.charge_after_persistent_fork();
        let custody = if matches!(self.storage, PersistentHashMapStorage::Exclusive(_)) {
            resources.map(|resources| {
                let growth = charge
                    .expect("reserved fork was prepared")
                    .checked_sub(self.retained_charge.expect("prepared source charge"))
                    .expect("fork conversion only adds storage");
                resources
                    .split_embedded(growth)
                    .expect("complete conversion was reserved")
            })
        } else {
            None
        };
        self.retained_charge = None;
        if let PersistentHashMapStorage::Exclusive(values) = &mut self.storage {
            let base = Arc::new(RetainedStorageBacking::new(std::mem::take(values), custody));
            let len = base.len();
            self.storage = PersistentHashMapStorage::ForkShared {
                base,
                changes: im::HashMap::new(),
                collision_extents: Some(CollisionExtents::default()),
                len,
            };
        }
        self.retained_charge = charge;
        self.fork_storage_identity()
    }
}
