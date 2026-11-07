//! Native entity image projection retains its complete source owner.
use super::*;
impl CheckpointArenaKind for EntityRecordKind {
    type ImageKind = EntityCheckpointImageKind;

    fn plan(
        catalog: &AspectContractPlanCatalog,
        kind_id: KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        catalog.entity_plans.get(&kind_id)
    }

    fn meta_kind(meta: &Self::Meta) -> KindId {
        meta.kind_id
    }

    fn meta_kind_from_image(meta: &VersionedEntityMetadataImage) -> KindId {
        meta.kind_id
    }

    fn extra_to_image(
        extra: &Self::Extra,
        contracts: Option<&CheckpointAspectContractCatalog>,
    ) -> Result<EntityExtraImage, DurabilityError> {
        Ok(EntityExtraImage {
            structural_fingerprint: extra.structural_fingerprint,
            lineage_id: extra.lineage_id,
            authoritative_aspect_state: export_state(
                extra.authoritative_aspect_state(),
                contracts,
            )?,
        })
    }

    fn extra_from_image_using(
        extra: EntityExtraImage,
        contracts: &CheckpointAspectContractCatalog,
        admission: &ImageReadmission<'_, '_, '_>,
    ) -> Result<Self::Extra, crate::durability::data::RelationalRecoveryStop> {
        let (state, memory) = readmit_state_using(
            extra.authoritative_aspect_state,
            contracts,
            crate::storage::substrate::StorageAllocation::<EntityExtra>::layout_bytes(),
            admission,
        )?;
        Ok(EntityExtra::from_funded_parts(
            extra.structural_fingerprint,
            extra.lineage_id,
            state,
            memory,
        ))
    }

    fn meta_to_image(
        meta: &Self::Meta,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<VersionedEntityMetadataImage, DurabilityError> {
        Ok(VersionedEntityMetadataImage {
            effective_at: meta.effective_at,
            retired_at: meta.retired_at,
            generation: meta.generation,
            kind_id: meta.kind_id,
            lineage_id: meta.lineage_id,
            authoritative_aspect_state: export_state(
                meta.authoritative_aspect_state(),
                Some(contracts),
            )?,
        })
    }

    fn meta_from_image_using(
        meta: VersionedEntityMetadataImage,
        contracts: &CheckpointAspectContractCatalog,
        admission: &ImageReadmission<'_, '_, '_>,
    ) -> Result<Self::Meta, crate::durability::data::RelationalRecoveryStop> {
        let (state, memory) = readmit_state_using(
            meta.authoritative_aspect_state,
            contracts,
            crate::storage::substrate::StorageAllocation::<EntityExtra>::layout_bytes(),
            admission,
        )?;
        let extra = crate::storage::substrate::StorageAllocation::new(
            EntityExtra::from_funded_parts(None, meta.lineage_id, state, memory),
        );
        let value = VersionedEntityMetadata::from_ordinary(
            meta.effective_at,
            meta.retired_at,
            meta.generation,
            meta.kind_id,
            meta.lineage_id,
            extra,
        );
        match admission.request {
            Some(request) => value.fund_for_request(request).map_err(Into::into),
            None => Ok(value),
        }
    }
}
