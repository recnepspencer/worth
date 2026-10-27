//! Head-only entity metadata from already accepted native custody; never loads.
use super::VisibilityReadContext;
use crate::identity::data::{EntityId, KindId, PartitionId, VersionId};
use crate::snapshots::data::SnapshotHandle;
use crate::storage::residency::{Head, RelationalResidencyLease};
use crate::symbols::data::StringInterner;
use crate::visibility::materialization::read_records::RelationalEntityRetirement;
use worth_foundational::facade::AspectKey;

/// A selected region or its aspect-name dictionary has no resident witness.
/// This is a pin refusal, not a cold-source read failure: no source is opened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalEntityMetadataPinDenial {
    RegionNotResident(PartitionId),
    SymbolsNotResident(PartitionId),
}
/// Current native metadata only. Construction retains a counted Head witness
/// and this runtime's already admitted aspect-name meanings. No history or
/// payload access, implicit promotion, request, or runtime borrow is exposed.
/// ```compile_fail,E0599
/// use worth_relational::facade::runtime::RelationalPinnedEntityMetadata;
/// fn historical(pin: &RelationalPinnedEntityMetadata) { pin.full_view(); }
/// ```
#[derive(Clone, Debug)]
pub struct RelationalPinnedEntityMetadata {
    entity: EntityId,
    version: VersionId,
    region: Option<RelationalResidencyLease<Head>>,
    symbols: StringInterner,
}
impl RelationalPinnedEntityMetadata {
    pub fn live_kind(&self) -> Option<KindId> {
        let slot = self
            .region
            .as_ref()?
            .head_view()
            .entity_arena()
            .get(&self.entity)?;
        slot.is_live().then(|| slot.kind_id()).flatten()
    }
    pub fn aspect_version(&self, aspect: &AspectKey) -> Option<Option<u64>> {
        let head = self.region.as_ref()?.head_view();
        let slot = head.entity_arena().get(&self.entity)?;
        if !slot.is_live() {
            return None;
        }
        let versions = head
            .entity_arena()
            .aspect_versions_at(self.entity.slot_index())?;
        Some(
            self.symbols
                .symbol(aspect.as_str())
                .and_then(|symbol| versions.get(&symbol).copied()),
        )
    }
    pub fn retirement(&self) -> Option<RelationalEntityRetirement> {
        RelationalEntityRetirement::from_head(
            self.region.as_ref()?.head_view(),
            self.entity,
            self.version,
        )
    }
}
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
        let Some(root) = self
            .runtime
            .visibility
            .selected_snapshot_root(handle.snapshot_id)
        else {
            return Ok(None);
        };
        let region = root
            .accepted_head_region_witness(entity.partition_id)
            .map_err(RelationalEntityMetadataPinDenial::RegionNotResident)?;
        // A metadata pin needs the selected record's aspect-name closure, not
        // unrelated field texts or buckets. Check admitted meanings without I/O.
        let symbols = self.runtime.services.symbols.with_read(|symbols| {
            if let Some(head) = region.as_ref().map(|lease| lease.head_view())
                && let Some(versions) = head.entity_arena().aspect_versions_at(entity.slot_index())
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
        Ok(Some(RelationalPinnedEntityMetadata {
            entity,
            version: handle.version_id,
            region,
            symbols,
        }))
    }
}
