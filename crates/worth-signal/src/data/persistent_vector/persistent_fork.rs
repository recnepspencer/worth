use super::{PersistentVector, PersistentVectorStorage};
use crate::data::retained_storage::{
    RetainedStorageBacking, SignalConditionalRetentionReservation,
};
use std::sync::Arc;

impl<T: Clone, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
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
        let fork_charge = self.charge_after_persistent_fork();
        let custody = if matches!(self.storage, PersistentVectorStorage::Exclusive(_)) {
            resources.map(|resources| {
                let growth = fork_charge
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
        if let PersistentVectorStorage::Exclusive(values) = &mut self.storage {
            let base = Arc::new(RetainedStorageBacking::new(std::mem::take(values), custody));
            let len = base.len();
            self.storage = PersistentVectorStorage::ForkShared {
                base,
                changed_pages: im::OrdMap::new(),
                len,
            };
        }
        self.retained_charge = fork_charge;
        match &self.storage {
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                len,
            } => Self {
                retained_charge: fork_charge,
                storage: PersistentVectorStorage::ForkShared {
                    base: Arc::clone(base),
                    changed_pages: changed_pages.clone(),
                    len: *len,
                },
            },
            PersistentVectorStorage::Exclusive(_) => unreachable!("fork preparation must share"),
        }
    }
}
