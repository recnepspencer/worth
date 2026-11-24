//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl VisibilityReadContext<'_> {
    /// Clone only a selected root's already accepted Head custody. This door
    /// performs no demand or storage read. An unavailable handle is genuine
    /// absence; an existing cold region is a typed refusal. Later transport
    /// damage cannot change the immutable contents this custody accepted.
    pub fn pin_resident_entity_metadata(
        &self,
        handle: &SnapshotHandle,
        entity: EntityId,
    ) -> Result<Option<RelationalPinnedEntityMetadata>, RelationalEntityMetadataPinDenial> {
        if handle.runtime_instance_id != self.runtime.runtime_instance_id() {
            return Ok(None);
        }
        self.runtime
            .visibility
            .with_selected_snapshot_head(handle.snapshot_id, entity.partition_id, |head| {
                let region = head
                    .witness()
                    .map_err(RelationalEntityMetadataPinDenial::RegionNotResident)?;
                // A metadata pin needs the selected record's aspect-name closure, not
                // unrelated field texts or buckets. Check admitted meanings without I/O.
                let symbols = self.runtime.services.symbols.with_read(|symbols| {
                    if let Some(head) = region.as_ref().map(|lease| lease.head_view())
                        && let Some(versions) =
                            head.entity_arena().aspect_versions_at(entity.slot_index())
                        && versions
                            .keys()
                            .any(|symbol| symbols.resolve(*symbol).is_none())
                    {
                        return Err(RelationalEntityMetadataPinDenial::SymbolsNotResident(
                            entity.partition_id,
                        ));
                    }
                    Ok(symbols.clone())
                })?;
                Ok(RelationalPinnedEntityMetadata {
                    entity,
                    version: handle.version_id,
                    region,
                    symbols,
                })
            })
            .transpose()
    }
}
