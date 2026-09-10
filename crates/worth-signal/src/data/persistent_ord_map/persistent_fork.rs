use super::{PersistentOrdMap, PersistentOrdMapStorage};
use crate::data::retained_storage::{
    RetainedStorageBacking, SignalConditionalRetentionReservation,
};
use std::sync::Arc;

impl<K: Clone + Ord, V: Clone> PersistentOrdMap<K, V> {
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
        let custody = if matches!(self.storage, PersistentOrdMapStorage::Exclusive(_)) {
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
        if let PersistentOrdMapStorage::Exclusive(values) = &mut self.storage {
            let base = Arc::new(RetainedStorageBacking::new(std::mem::take(values), custody));
            let len = base.len();
            self.storage = PersistentOrdMapStorage::ForkShared {
                base,
                changes: im::OrdMap::new(),
                retired_base_intervals: im::OrdMap::new(),
                len,
            };
        }
        self.retained_charge = charge;
        self.fork_storage_identity()
    }
}
