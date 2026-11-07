//! Native relation image projection retains its complete source owner.
use super::*;
impl CheckpointArenaKind for RelationRecordKind {
    type ImageKind = RelationCheckpointImageKind;

    fn plan(
        catalog: &AspectContractPlanCatalog,
        kind_id: KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        catalog.relation_plans.get(&kind_id)
    }

    fn meta_kind(meta: &Self::Meta) -> KindId {
        meta.kind_id
    }

    fn meta_kind_from_image(meta: &VersionedRelationMetadataImage) -> KindId {
        meta.kind_id
    }

    fn extra_to_image(
        extra: &Self::Extra,
        contracts: Option<&CheckpointAspectContractCatalog>,
    ) -> Result<RelationExtraImage, DurabilityError> {
        Ok(RelationExtraImage {
            endpoints: extra
                .endpoints
                .as_ref()
                .map(|endpoints| RelationEndpointsImage {
                    source: endpoints.source,
                    target: endpoints.target,
                }),
            authoritative_aspect_state: export_state(
                extra.authoritative_aspect_state(),
                contracts,
            )?,
        })
    }

    fn extra_from_image_using(
        extra: RelationExtraImage,
        contracts: &CheckpointAspectContractCatalog,
        admission: &ImageReadmission<'_, '_, '_>,
    ) -> Result<Self::Extra, crate::durability::data::RelationalRecoveryStop> {
        let (state, memory) =
            readmit_state_using(
                extra.authoritative_aspect_state,
                contracts,
                crate::storage::substrate::StorageAllocation::<
                    crate::storage::substrate::RelationExtra,
                >::layout_bytes(),
                admission,
            )?;
        Ok(crate::storage::substrate::RelationExtra::from_funded_parts(
            extra.endpoints.map(|value| RelationEndpoints {
                source: value.source,
                target: value.target,
            }),
            state,
            memory,
        ))
    }

    fn meta_to_image(
        meta: &Self::Meta,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<VersionedRelationMetadataImage, DurabilityError> {
        Ok(VersionedRelationMetadataImage {
            effective_at: meta.effective_at,
            retired_at: meta.retired_at,
            generation: meta.generation,
            kind_id: meta.kind_id,
            endpoints: RelationEndpointsImage {
                source: meta.endpoints.source,
                target: meta.endpoints.target,
            },
            authoritative_aspect_state: export_state(
                meta.authoritative_aspect_state(),
                Some(contracts),
            )?,
        })
    }

    fn meta_from_image_using(
        meta: VersionedRelationMetadataImage,
        contracts: &CheckpointAspectContractCatalog,
        admission: &ImageReadmission<'_, '_, '_>,
    ) -> Result<Self::Meta, crate::durability::data::RelationalRecoveryStop> {
        let (state, memory) =
            readmit_state_using(
                meta.authoritative_aspect_state,
                contracts,
                crate::storage::substrate::StorageAllocation::<
                    crate::storage::substrate::RelationExtra,
                >::layout_bytes(),
                admission,
            )?;
        let endpoints = RelationEndpoints {
            source: meta.endpoints.source,
            target: meta.endpoints.target,
        };
        let extra = crate::storage::substrate::StorageAllocation::new(
            crate::storage::substrate::RelationExtra::from_funded_parts(
                Some(endpoints.clone()),
                state,
                memory,
            ),
        );
        let value = VersionedRelationMetadata::from_ordinary(
            meta.effective_at,
            meta.retired_at,
            meta.generation,
            meta.kind_id,
            endpoints,
            extra,
        );
        match admission.request {
            Some(request) => value.fund_for_request(request).map_err(Into::into),
            None => Ok(value),
        }
    }
}
