//! Request-admitted maintenance edits of already Full resident mirror targets.
use super::{PartitionEditionWriter, PartitionState};
use crate::identity::data::PartitionId;

pub(crate) enum TryPartitionMutationDenied<E> {
    Observed,
    SizeOverflow,
    Storage(crate::storage::data::RelationalStorageUnavailable),
    Admission(E),
}

impl PartitionEditionWriter<'_> {
    pub(crate) fn try_partition_mut<E>(
        &mut self,
        id: PartitionId,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<Option<&mut PartitionState>, TryPartitionMutationDenied<E>> {
        let map = self
            .mirror
            .edition
            .unobserved_map_mut()
            .ok_or(TryPartitionMutationDenied::Observed)?;
        let navigation = map.navigation_work_bound() as u64;
        prepare(navigation, 0).map_err(TryPartitionMutationDenied::Admission)?;
        if !map.contains_key(&id) {
            return Ok(None);
        }
        // Removing the selected key can detach shared AVL paths even when
        // this outer edition Arc is unique. Fund remove/reinsert coexistence
        // and the new retention value before editing. Payload admission follows.
        let paths = map
            .adoption_allocation_bound()
            .and_then(|bytes| bytes.checked_mul(2))
            .and_then(|bytes| {
                bytes.checked_add(crate::storage::substrate::StorageAllocation::<
                    crate::storage::residency::RelationalRegionRetention,
                >::layout_bytes())
            })
            .ok_or(TryPartitionMutationDenied::SizeOverflow)?;
        let partition = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<PartitionState>())
            .map_err(|_| TryPartitionMutationDenied::SizeOverflow)?
            .0
            .pad_to_align()
            .size() as u64;
        let bytes = paths
            .checked_add(partition)
            .ok_or(TryPartitionMutationDenied::SizeOverflow)?;
        prepare(1, bytes).map_err(TryPartitionMutationDenied::Admission)?;
        if !self.edits.contains_key(&id) {
            let target=self.mirror.admit_target(id).map_err(TryPartitionMutationDenied::Storage)?
                .ok_or_else(||TryPartitionMutationDenied::Storage(crate::storage::data::RelationalStorageUnavailable {
                    subject:crate::storage::data::RelationalStorageSubject::Region(id),
                    cause:crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
                }))?;
            if !self.begin_target_edit(&target) {
                return Err(TryPartitionMutationDenied::Storage(crate::storage::data::RelationalStorageUnavailable {
                    subject:crate::storage::data::RelationalStorageSubject::Region(id),
                    cause:crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
                }));
            }
        }
        Ok(self
            .edits
            .get_mut(&id)
            .map(|entry| entry.edit.partition_mut()))
    }
}
