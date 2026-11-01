//! Lineage replaces the whole funded extra instead of implicitly cloning it.
use super::RecordArena;
use crate::identity::data::LineageId;
use crate::storage::substrate::{EntityRecordKind, StorageAllocation};
#[derive(Debug)]
pub(crate) enum ExtraLineageDenial {
    Request(crate::execution::RelationalExecutionDenialCause),
    Missing(&'static str),
}
impl RecordArena<EntityRecordKind> {
    pub(crate) fn install_lineage_id(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        slot: usize,
        lineage: LineageId,
    ) -> Result<(), ExtraLineageDenial> {
        let physical = self
            .physical_index(slot)
            .ok_or(ExtraLineageDenial::Missing(
                "planned lineage entity slot is unavailable",
            ))?;
        let previous = self
            .extra
            .retained_value(physical)
            .ok_or(ExtraLineageDenial::Missing(
                "planned lineage entity metadata is unavailable",
            ))?;
        if self
            .metadata_history
            .get(physical)
            .and_then(|history| {
                history
                    .len()
                    .checked_sub(1)
                    .and_then(|index| history.get(index))
            })
            .is_none()
        {
            return Err(ExtraLineageDenial::Missing(
                "planned lineage metadata history is unavailable",
            ));
        }
        let mut replacement = previous
            .copy_for_request(request)
            .map_err(ExtraLineageDenial::Request)?;
        replacement.lineage_id = Some(lineage);
        let replacement = StorageAllocation::new(replacement);
        let metadata = self.metadata_history[physical]
            .last_mut()
            // Proof: the preflight retained its latest metadata; this shared borrow cannot remove it.
            .expect("the preflight retained its latest metadata");
        metadata.lineage_id = Some(lineage);
        metadata.extra = replacement.clone();
        self.extra.set_shared(physical, replacement);
        Ok(())
    }
}
