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

    fn extra_from_image(
        extra: RelationExtraImage,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<Self::Extra, DurabilityError> {
        Ok(
            crate::storage::substrate::RelationExtra::from_unbudgeted_parts(
                extra.endpoints.as_ref().map(|endpoints| RelationEndpoints {
                    source: endpoints.source,
                    target: endpoints.target,
                }),
                readmit_state(extra.authoritative_aspect_state, contracts)?,
            ),
        )
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

    fn meta_from_image(
        meta: VersionedRelationMetadataImage,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<Self::Meta, DurabilityError> {
        Ok(VersionedRelationMetadata::from_ordinary(
            meta.effective_at,
            meta.retired_at,
            meta.generation,
            meta.kind_id,
            RelationEndpoints {
                source: meta.endpoints.source,
                target: meta.endpoints.target,
            },
            crate::storage::substrate::StorageAllocation::new(
                crate::storage::substrate::RelationExtra::from_unbudgeted_parts(
                    Some(RelationEndpoints {
                        source: meta.endpoints.source,
                        target: meta.endpoints.target,
                    }),
                    readmit_state(meta.authoritative_aspect_state, contracts)?,
                ),
            ),
        ))
    }
}
