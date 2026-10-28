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
        prepare(map.len() as u64, 0).map_err(TryPartitionMutationDenied::Admission)?;
        if !map.contains_key(&id) {
            return Ok(None);
        }
        // The resident edit can detach its shared partition owner. Value paths
        // and arena navigation are admitted separately by the retention step.
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<PartitionState>())
            .map_err(|_| TryPartitionMutationDenied::SizeOverflow)?
            .0
            .pad_to_align()
            .size() as u64;
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
