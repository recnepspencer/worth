//! Select a retained version in its original slot; this never allocates an identity.
use super::*;
use crate::storage::substrate::HistoricalMetadata;

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn select_retained_version(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        id: &RecordId<K::Domain>,
        source: super::SlotView<'_, K>,
        version: VersionId,
    ) -> Result<(), MetadataWriteDenial> {
        self.select_retained_version_using(Some(request), id, source, version)
    }

    fn select_retained_version_using(
        &mut self,
        request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        id: &RecordId<K::Domain>,
        source: super::SlotView<'_, K>,
        version: VersionId,
    ) -> Result<(), MetadataWriteDenial> {
        if let Some(request) = request {
            request
                .consult()
                .map_err(|cause| MetadataWriteDenial::Request(cause.into()))?;
        }
        let prepare = &mut |metadata: K::Meta| match request {
            Some(request) => metadata.fund_for_request(request),
            None => Ok(metadata),
        };
        let slot = crate::storage::substrate::slot_of::<K>(id);
        if !source.is_live() || source.record_id() != *id {
            return Err(MetadataWriteDenial::Invalid(
                "historical selection requires its exact retained live source",
            ));
        }
        let birth = source.created_at_version();
        let kind = source.kind_id().ok_or("selected record kind is absent")?;
        let generation = crate::storage::substrate::generation_of::<K>(id);
        let partition = crate::storage::substrate::partition_of::<K>(id);
        if let Some(physical) = self.physical_index(slot) {
            if self.generations[physical] != generation
                || self.partition_ids[physical] != partition
                || self.kind_ids[physical] != Some(kind)
                || self.created_at[physical] != birth
                || self.lifecycle[physical] == RecordLifecycleState::Reusable
            {
                return Err(MetadataWriteDenial::Invalid(
                    "historical selection cannot replace another slot identity",
                ));
            }
            let extra = match request {
                Some(request) => K::copy_extra_for_request(source.extra(), request)?,
                None => source.extra().clone(),
            };
            self.apply_extra_update_using(slot, extra, version, request, prepare)?
                .ok_or("selected record cannot issue fresh native metadata")?;
            if self.lifecycle[physical] == RecordLifecycleState::MaterializationUnavailable {
                self.field_revisions.set_value_using(
                    physical,
                    Some(Default::default()),
                    request,
                )?;
            }
            self.lifecycle
                .set_value_using(physical, RecordLifecycleState::Live, request)?;
            self.retired_at.set_value_using(physical, None, request)?;
            super::slot_lifecycle::set_slot_presence(
                &mut self.live_bitset,
                &mut self.reclaimable_bitset,
                slot,
                true,
                false,
                request,
            )?;
        } else {
            // A sibling branch may never have materialized this globally issued
            // slot. The retained source supplies its original identity and birth.
            let extra = match request {
                Some(request) => K::copy_extra_for_request(source.extra(), request)?,
                None => source.extra().clone(),
            };
            self.write_reserved_slot_using(
                SlotInit {
                    partition_id: partition,
                    kind_id: kind,
                    version_id: version,
                    extra,
                },
                slot,
                generation,
                request,
                prepare,
            )?;
            let physical = self
                .physical_index(slot)
                .ok_or("selected slot disappeared")?;
            self.created_at.set_value_using(physical, birth, request)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod payload_tests;
