use super::{FlatSegments, SegmentedStorage, SegmentedStore};
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageBacking, SignalConditionalRetentionReservation,
};
use std::marker::PhantomData;
use std::sync::Arc;

impl<T: Clone, Id: Clone> SegmentedStore<T, Id> {
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
        mut resources: Option<&mut SignalConditionalRetentionReservation>,
    ) -> Self {
        if let SegmentedStorage::Exclusive(flat) = &mut self.storage {
            let custody = resources.as_deref_mut().map(|resources| {
                resources
                    .split_embedded(
                        arc_allocation_charge::<RetainedStorageBacking<FlatSegments<T>>>()
                            .expect("fixed backing size fits"),
                    )
                    .expect("complete flat conversion was reserved")
            });
            let base = Arc::new(RetainedStorageBacking::new(
                FlatSegments {
                    items: std::mem::take(&mut flat.items),
                    segments: std::mem::take(&mut flat.segments),
                },
                custody,
            ));
            self.storage = SegmentedStorage::ForkShared {
                base,
                appended: crate::data::persistent_vector::PersistentVector::new(),
            };
        }
        let storage = match &mut self.storage {
            SegmentedStorage::ForkShared { base, appended } => SegmentedStorage::ForkShared {
                base: Arc::clone(base),
                appended: match resources.as_deref_mut() {
                    Some(resources) => appended.fork_reserved(resources),
                    None => appended.fork_persistent(),
                },
            },
            SegmentedStorage::Exclusive(_) => unreachable!("fork converts segmented storage"),
        };
        Self {
            storage,
            interner: match resources {
                Some(resources) => self.interner.fork_reserved(resources),
                None => self.interner.fork_persistent(),
            },
            id: PhantomData,
        }
    }
}
