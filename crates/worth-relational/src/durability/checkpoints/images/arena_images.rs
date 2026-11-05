mod readmission;
pub(in crate::durability::checkpoints) use readmission::ImageReadmission;
mod entity;
mod relation;
use std::marker::PhantomData;

use crate::durability::data::{
    DurabilityError, DurableBitSet, EntityCheckpointImageKind, EntityExtraImage,
    RecordArenaCheckpointImage, RecordArenaCheckpointKind, RecoveryFailureClass,
    RelationCheckpointImageKind, RelationEndpointsImage, RelationExtraImage,
    VersionedEntityMetadataImage, VersionedRelationMetadataImage,
};
use crate::identity::data::{KindId, PartitionId};
use crate::schema::data::{AspectContractPlanCatalog, LoweredAspectContractPlan};
use crate::storage::substrate::{
    EntityExtra, EntityRecordKind, RecordArena, RecordKind, RelationEndpoints, RelationRecordKind,
    VersionedEntityMetadata, VersionedRelationMetadata,
};

use super::bitset_image::restore_bitset;
use crate::durability::checkpoints::aspect_state_images::{
    export_state, readmit_state_using, CheckpointAspectContractCatalog,
};

pub(super) trait CheckpointArenaKind: RecordKind {
    type ImageKind: RecordArenaCheckpointKind;

    fn plan(
        catalog: &AspectContractPlanCatalog,
        kind_id: KindId,
    ) -> Option<&LoweredAspectContractPlan>;
    fn meta_kind(meta: &Self::Meta) -> KindId;
    fn meta_kind_from_image(
        meta: &<Self::ImageKind as RecordArenaCheckpointKind>::MetaImage,
    ) -> KindId;
    fn extra_to_image(
        extra: &Self::Extra,
        contracts: Option<&CheckpointAspectContractCatalog>,
    ) -> Result<<Self::ImageKind as RecordArenaCheckpointKind>::ExtraImage, DurabilityError>;
    fn extra_from_image_using(
        extra: <Self::ImageKind as RecordArenaCheckpointKind>::ExtraImage,
        contracts: &CheckpointAspectContractCatalog,
        admission: &ImageReadmission<'_, '_, '_>,
    ) -> Result<Self::Extra, crate::durability::data::RelationalRecoveryStop>;
    fn meta_to_image(
        meta: &Self::Meta,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<<Self::ImageKind as RecordArenaCheckpointKind>::MetaImage, DurabilityError>;
    fn meta_from_image_using(
        meta: <Self::ImageKind as RecordArenaCheckpointKind>::MetaImage,
        contracts: &CheckpointAspectContractCatalog,
        admission: &ImageReadmission<'_, '_, '_>,
    ) -> Result<Self::Meta, crate::durability::data::RelationalRecoveryStop>;
}

fn missing_plan(kind_id: KindId) -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::SchemaMismatch,
        format!(
            "checkpoint record kind {} has no lowered aspect plan",
            kind_id.0
        ),
    )
}

pub(super) fn arena_to_image<K: CheckpointArenaKind>(
    arena: RecordArena<K>,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
) -> Result<RecordArenaCheckpointImage<K::ImageKind>, DurabilityError> {
    let metadata_history = arena
        .metadata_history
        .iter()
        .map(|entries| {
            entries
                .iter()
                .map(|meta| {
                    let kind_id = K::meta_kind(meta);
                    K::plan(catalog, kind_id).ok_or_else(|| missing_plan(kind_id))?;
                    K::meta_to_image(meta, contracts)
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let extra = arena
        .extra
        .iter()
        .zip(arena.kind_ids.iter().copied())
        .map(|(extra, kind_id)| {
            let contracts = kind_id
                .and_then(|kind_id| K::plan(catalog, kind_id))
                .map(|_| contracts);
            K::extra_to_image(extra, contracts)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RecordArenaCheckpointImage {
        slots: arena.slots.slots().to_vec(),
        generations: arena.generations.into_vec(),
        lifecycle: arena.lifecycle.into_vec(),
        kind_ids: arena.kind_ids.into_vec(),
        metadata_history,
        created_at: arena.created_at.into_vec(),
        retired_at: arena.retired_at.into_vec(),
        // These are ordinary owned wire projections, outside Complete emission.
        aspect_versions: arena
            .aspect_versions
            .into_vec()
            .into_iter()
            .map(|rows| rows.iter().map(|(key, value)| (*key, *value)).collect())
            .collect(),
        field_revisions: arena
            .field_revisions
            .into_vec()
            .into_iter()
            .map(|rows| rows.map(|rows| rows.iter().map(|(key, value)| (*key, *value)).collect()))
            .collect(),
        extra,
        diagnostics_enrichment: arena.diagnostics_enrichment.into_vec(),
        branch_pins: arena.branch_pins.into_vec(),
        replay_pins: arena.replay_pins.into_vec(),
        snapshot_pins: arena.snapshot_pins.into_vec(),
        live_bitset: DurableBitSet {
            words: Vec::new(),
            sparse_words: arena.live_bitset.sparse_words(),
        },
        reclaimable_bitset: DurableBitSet {
            words: Vec::new(),
            sparse_words: arena.reclaimable_bitset.sparse_words(),
        },
        free_list: Vec::new(),
        marker: PhantomData,
    })
}

pub(super) fn arena_from_image<K: CheckpointArenaKind>(
    partition_id: PartitionId,
    image: RecordArenaCheckpointImage<K::ImageKind>,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
    admission: &ImageReadmission<'_, '_, '_>,
) -> Result<RecordArena<K>, crate::durability::data::RelationalRecoveryStop> {
    readmission::arena::<K>(partition_id, image, catalog, contracts, admission)
}
