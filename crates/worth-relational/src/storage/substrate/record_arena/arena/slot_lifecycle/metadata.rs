//! Original-request metadata preparation precedes every target history edit.
use super::*;
use crate::storage::substrate::HistoricalMetadata;
use worth_execution::ExecutionRequest;
#[derive(Debug)]
pub(crate) enum MetadataWriteDenial {
    Request(Cause),
    Invalid(&'static str),
}
impl From<Cause> for MetadataWriteDenial {
    fn from(cause: Cause) -> Self {
        Self::Request(cause)
    }
}
impl From<&'static str> for MetadataWriteDenial {
    fn from(detail: &'static str) -> Self {
        Self::Invalid(detail)
    }
}
#[cfg(test)]
pub(super) fn ordinary_denial(denial: MetadataWriteDenial) -> &'static str {
    match denial {
        MetadataWriteDenial::Invalid(detail) => detail,
        MetadataWriteDenial::Request(_) => {
            unreachable!("the ordinary metadata identity callback cannot refuse")
        }
    }
}
impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn write_reserved_slot_for_request(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        init: SlotInit<K>,
        slot: usize,
        generation: u32,
    ) -> Result<bool, MetadataWriteDenial> {
        self.write_reserved_slot_using(init, slot, generation, Some(request), &mut |metadata| {
            metadata.fund_for_request(request)
        })
    }
    /// Existing ordinary fixture/maintenance entry; its independent metadata is unqualified.
    #[cfg(test)]
    pub(crate) fn write_reserved_slot(
        &mut self,
        init: SlotInit<K>,
        slot: usize,
        generation: u32,
    ) -> Result<bool, &'static str> {
        self.write_reserved_slot_using(init, slot, generation, None, &mut Ok)
            .map_err(ordinary_denial)
    }
    pub(crate) fn rematerialize_for_request(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        id: &crate::identity::data::RecordId<K::Domain>,
        kind_id: crate::identity::data::KindId,
        version_id: VersionId,
        extra: K::Extra,
    ) -> Result<(), MetadataWriteDenial> {
        self.rematerialize_using(
            id,
            kind_id,
            version_id,
            extra,
            Some(request),
            &mut |metadata| metadata.fund_for_request(request),
        )
    }
    /// Existing ordinary fixture/maintenance entry; its independent metadata is unqualified.
    #[cfg(test)]
    pub(crate) fn rematerialize(
        &mut self,
        id: &crate::identity::data::RecordId<K::Domain>,
        kind_id: crate::identity::data::KindId,
        version_id: VersionId,
        extra: K::Extra,
    ) -> Result<(), &'static str> {
        self.rematerialize_using(id, kind_id, version_id, extra, None, &mut Ok)
            .map_err(ordinary_denial)
    }
    pub(crate) fn apply_extra_update_for_request(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        slot: usize,
        extra: K::Extra,
        version_id: VersionId,
    ) -> Result<Option<()>, Cause> {
        self.apply_extra_update_using(slot, extra, version_id, Some(request), &mut |metadata| {
            metadata.fund_for_request(request)
        })
    }
    /// Existing ordinary fixture/maintenance entry; its independent metadata is unqualified.
    #[cfg(test)]
    pub(crate) fn apply_extra_update(
        &mut self,
        slot: usize,
        extra: K::Extra,
        version_id: VersionId,
    ) -> Option<()> {
        self.apply_extra_update_using(slot, extra, version_id, None, &mut Ok)
            .expect("the ordinary metadata identity callback cannot refuse")
    }
    pub(crate) fn retire_for_request(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        slot: usize,
        version_id: VersionId,
    ) -> Result<Option<()>, Cause> {
        self.retire_using(slot, version_id, Some(request), &mut |metadata| {
            metadata.fund_for_request(request)
        })
    }
    /// Existing ordinary fixture/maintenance entry; its independent metadata is unqualified.
    #[cfg(test)]
    pub(crate) fn retire(&mut self, slot: usize, version_id: VersionId) -> Option<()> {
        self.retire_using(slot, version_id, None, &mut Ok)
            .expect("the ordinary metadata identity callback cannot refuse")
    }
}
