//! Prepare and install a retired suffix row before changing its owner.
use super::*;
use worth_execution::ExecutionRequest;
impl<K: RecordKind> CurrentSlotData<'_, K> {
    pub(super) fn prepare_retired_metadata(
        &self,
        physical: usize,
        version: VersionId,
        prepare: &mut impl FnMut(K::Meta) -> Result<K::Meta, Cause>,
    ) -> Result<Option<(usize, K::Meta)>, Cause> {
        let history = &self.metadata_history[physical];
        let Some(index) = history.len().checked_sub(1) else {
            return Ok(None);
        };
        // Clone copies only inline metadata and the existing Extra handle, not a heap.
        let mut replacement = history[index].clone();
        K::retire_metadata(&mut replacement, version);
        Ok(Some((index, prepare(replacement)?)))
    }
    pub(super) fn install_retired_metadata(
        &mut self,
        physical: usize,
        prepared: Option<(usize, K::Meta)>,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<(), Cause> {
        if let Some((index, metadata)) = prepared {
            self.metadata_history
                .carrier_mut_using(physical, request)?
                .expect("existing metadata history")
                .set_value_using(index, metadata, request)?;
        }
        Ok(())
    }
}
