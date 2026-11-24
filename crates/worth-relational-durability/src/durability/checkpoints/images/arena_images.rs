pub mod readmission;
pub(in crate::internal_port::durability::checkpoints) use readmission::ImageReadmission;
pub mod entity;
pub mod relation;
use std::marker::PhantomData;

use worth_relational_contracts::internal_port::durability::data::recovery_errors::{
    DurabilityError, RecoveryFailureClass,
};
use worth_relational_contracts::internal_port::identity::data::primitives::{KindId, PartitionId};
use worth_relational_contracts::internal_port::schema::data::aspect_contracts::lowered_plan::{
    AspectContractPlanCatalog, LoweredAspectContractPlan,
};
use worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::{
    DurableBitSet, EntityCheckpointImageKind, EntityExtraImage, RecordArenaCheckpointImage,
    RecordArenaCheckpointKind, RelationCheckpointImageKind, RelationEndpointsImage,
    RelationExtraImage, VersionedEntityMetadataImage, VersionedRelationMetadataImage,
};
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::RecordArena;
use worth_relational_storage::internal_port::storage::substrate::record_arena::kinds::{
    EntityRecordKind, RecordKind, RelationRecordKind,
};
use worth_relational_storage::internal_port::storage::substrate::record_arena::values::extra::EntityExtra;
use worth_relational_storage::internal_port::storage::substrate::record_arena::values::metadata::{
    VersionedEntityMetadata, VersionedRelationMetadata,
};
use worth_relational_storage::internal_port::storage::substrate::record_arena::values::RelationEndpoints;

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
    ) -> Result<Self::Extra, worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop>;
    fn meta_to_image(
        meta: &Self::Meta,
        contracts: &CheckpointAspectContractCatalog,
    ) -> Result<<Self::ImageKind as RecordArenaCheckpointKind>::MetaImage, DurabilityError>;
    fn meta_from_image_using(
        meta: <Self::ImageKind as RecordArenaCheckpointKind>::MetaImage,
        contracts: &CheckpointAspectContractCatalog,
        admission: &ImageReadmission<'_, '_, '_>,
    ) -> Result<Self::Meta, worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop>;
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
    _lane: &worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
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
    lane: &worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    partition_id: PartitionId,
    image: RecordArenaCheckpointImage<K::ImageKind>,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
    admission: &ImageReadmission<'_, '_, '_>,
) -> Result<RecordArena<K>, worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop>{
    readmission::arena::<K>(lane, partition_id, image, catalog, contracts, admission)
}
