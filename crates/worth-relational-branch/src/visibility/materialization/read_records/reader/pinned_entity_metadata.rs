//! Head-only entity metadata from already accepted native custody; never loads.
use crate::identity::data::{EntityId, KindId, PartitionId, VersionId};
use crate::storage::residency::{Head, RelationalResidencyLease};
use crate::symbols::data::StringInterner;
use crate::visibility::materialization::read_records::projection::RelationalEntityRetirement;
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
#[cfg_attr(not(doctest), doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/facade_examples/pinned_entity_metadata.md")))]
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

pub fn assemble_pinned_entity_metadata(
    entity: EntityId,
    version: VersionId,
    region: Option<RelationalResidencyLease<Head>>,
    symbols: StringInterner,
) -> RelationalPinnedEntityMetadata {
    RelationalPinnedEntityMetadata {
        entity,
        version,
        region,
        symbols,
    }
}
