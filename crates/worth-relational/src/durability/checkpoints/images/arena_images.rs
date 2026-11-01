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
    export_state, readmit_state, CheckpointAspectContractCatalog,
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
    fn extra_from_image(
        extra: <Self::ImageKind as RecordArenaCheckpointKind>::ExtraImage,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<Self::Extra, DurabilityError>;
    fn meta_to_image(
        meta: &Self::Meta,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<<Self::ImageKind as RecordArenaCheckpointKind>::MetaImage, DurabilityError>;
    fn meta_from_image(
        meta: <Self::ImageKind as RecordArenaCheckpointKind>::MetaImage,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<Self::Meta, DurabilityError>;
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
                    let kind_id = K::meta_kind(&meta);
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
        aspect_versions: arena.aspect_versions.into_vec(),
        field_revisions: arena.field_revisions.into_vec(),
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
    mut image: RecordArenaCheckpointImage<K::ImageKind>,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
) -> Result<RecordArena<K>, DurabilityError> {
    let slots = if image.slots.is_empty() && !image.generations.is_empty() {
        (0..image.generations.len() as u64).collect()
    } else {
        std::mem::take(&mut image.slots)
    };
    if slots.len() != image.generations.len() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "record arena slot directory length differs from its SoA columns",
        ));
    }
    if image.field_revisions.len() != image.generations.len() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "record arena field revision length differs from its SoA columns",
        ));
    }
    let slots = crate::storage::substrate::RecordSlotDirectory::restore(slots)
        .map_err(|detail| DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail))?;
    let metadata_history = image
        .metadata_history
        .into_iter()
        .map(|entries| {
            entries
                .into_iter()
                .map(|meta| {
                    let kind_id = K::meta_kind_from_image(&meta);
                    K::plan(catalog, kind_id).ok_or_else(|| missing_plan(kind_id))?;
                    K::meta_from_image(meta, contracts)
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let extra = image
        .extra
        .into_iter()
        .zip(image.kind_ids.iter().copied())
        .map(|(extra, kind_id)| {
            if let Some(kind_id) = kind_id {
                K::plan(catalog, kind_id).ok_or_else(|| missing_plan(kind_id))?;
            }
            K::extra_from_image(extra, contracts)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RecordArena {
        slots,
        partition_ids: vec![partition_id; image.generations.len()].into(),
        history_prefixes: crate::storage::substrate::SharedColumn::with_default(
            image.generations.len(),
            None,
        ),
        generations: image.generations.into(),
        lifecycle: image.lifecycle.into(),
        kind_ids: image.kind_ids.into(),
        metadata_history: metadata_history.into_iter().map(Into::into).collect(),
        created_at: image.created_at.into(),
        retired_at: image.retired_at.into(),
        extra: extra.into(),
        aspect_versions: image.aspect_versions.into(),
        field_revisions: image.field_revisions.into(),
        diagnostics_enrichment: image.diagnostics_enrichment.into(),
        branch_pins: image.branch_pins.into(),
        replay_pins: image.replay_pins.into(),
        snapshot_pins: image.snapshot_pins.into(),
        live_bitset: restore_bitset(image.live_bitset),
        reclaimable_bitset: restore_bitset(image.reclaimable_bitset),
    })
}
