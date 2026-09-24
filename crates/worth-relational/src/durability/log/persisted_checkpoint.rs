use serde::{Deserialize, Serialize};

use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};

use super::local_store::DurableCheckpointFile;
use super::persisted_canonical_commit::PersistedCanonicalCommit;

#[derive(Serialize, Deserialize)]
pub(super) struct PersistedDurableCheckpointFile {
    checkpoint: PersistedDurableCheckpoint,
}

#[derive(Serialize, Deserialize)]
struct PersistedDurableCheckpoint {
    coverage: crate::durability::data::CheckpointCoverage,
    #[serde(default)]
    branch_cells: Vec<crate::branch::RelationalBranchCellCheckpoint>,
    #[serde(default)]
    branch_roots: Vec<crate::durability::data::DurableBranchRootImage>,
    #[serde(default)]
    branch_root_schema_images: Vec<crate::durability::data::DurableBranchRootSchemaImage>,
    #[serde(default)]
    record_identity: crate::durability::data::DurableRecordIdentityState,
    #[serde(default)]
    record_generation_high_water: Vec<crate::durability::data::DurableRecordGenerationHighWater>,
    #[serde(default)]
    reusable_record_slots: Vec<crate::durability::data::DurableReusableRecordSlot>,
    #[serde(default)]
    record_slot_frontiers: Vec<crate::durability::data::DurableRecordSlotFrontier>,
    envelopes: Vec<PersistedCanonicalCommit>,
    partition_images: Vec<crate::durability::data::PartitionCheckpointImage>,
    aspect_contracts: Vec<worth_foundational::facade::PortableAspectContract>,
    lineage: crate::lineage::data::LineageCheckpointArtifact,
    index_definitions: Vec<crate::indexes::data::DerivedIndexDefinition>,
    #[serde(default)]
    derived_index_artifacts: crate::indexes::data::DerivedIndexArtifacts,
    #[serde(default)]
    derived_index_checkpoint:
        Option<crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>,
    #[serde(default)]
    derived_index_checkpoint_format: u16,
    symbol_table: crate::symbols::data::SymbolTableSnapshot,
    runtime_name: String,
}

impl PersistedDurableCheckpointFile {
    pub(super) fn from_current(file: &DurableCheckpointFile) -> Self {
        let checkpoint = &file.checkpoint;
        Self {
            checkpoint: PersistedDurableCheckpoint {
                coverage: checkpoint.coverage.clone(),
                branch_cells: checkpoint.branch_cells.clone(),
                branch_roots: checkpoint.branch_roots.clone(),
                branch_root_schema_images: checkpoint.branch_root_schema_images.clone(),
                record_identity: checkpoint.record_identity.clone(),
                record_generation_high_water: checkpoint.record_generation_high_water.clone(),
                reusable_record_slots: checkpoint.reusable_record_slots.clone(),
                record_slot_frontiers: checkpoint.record_slot_frontiers.clone(),
                envelopes: checkpoint
                    .envelopes
                    .iter()
                    .map(PersistedCanonicalCommit::from_checkpoint_positioned)
                    .collect(),
                partition_images: checkpoint.partition_images.clone(),
                aspect_contracts: checkpoint.aspect_contracts.clone(),
                lineage: checkpoint.lineage.clone(),
                index_definitions: checkpoint.index_definitions.clone(),
                derived_index_artifacts: checkpoint.derived_index_artifacts.clone(),
                derived_index_checkpoint: checkpoint.derived_index_checkpoint.clone(),
                derived_index_checkpoint_format: checkpoint.derived_index_checkpoint_format,
                symbol_table: checkpoint.symbol_table.clone(),
                runtime_name: checkpoint.runtime_name.clone(),
            },
        }
    }

    pub(super) fn from_checkpoint(checkpoint: DurableCheckpoint) -> Self {
        let DurableCheckpoint {
            coverage,
            branch_cells,
            branch_roots,
            branch_root_schema_images,
            record_identity,
            record_generation_high_water,
            reusable_record_slots,
            record_slot_frontiers,
            envelopes,
            partition_images,
            aspect_contracts,
            lineage,
            index_definitions,
            derived_index_artifacts,
            derived_index_checkpoint,
            derived_index_checkpoint_format,
            symbol_table,
            runtime_name,
        } = checkpoint;
        Self {
            checkpoint: PersistedDurableCheckpoint {
                coverage,
                branch_cells,
                branch_roots,
                branch_root_schema_images,
                record_identity,
                record_generation_high_water,
                reusable_record_slots,
                record_slot_frontiers,
                envelopes: envelopes
                    .iter()
                    .map(PersistedCanonicalCommit::from_checkpoint_positioned)
                    .collect(),
                partition_images,
                aspect_contracts,
                lineage,
                index_definitions,
                derived_index_artifacts,
                derived_index_checkpoint,
                derived_index_checkpoint_format,
                symbol_table,
                runtime_name,
            },
        }
    }

    pub(super) fn readmit(self) -> Result<DurableCheckpointFile, DurabilityError> {
        let checkpoint = self.checkpoint;
        if !crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::supports_outer_format(
            checkpoint.derived_index_checkpoint_format,
            checkpoint.derived_index_checkpoint.is_some(),
        ) {
            return Err(DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "derived index checkpoint format or payload is missing",
            ));
        }
        let envelopes = checkpoint
            .envelopes
            .into_iter()
            .map(|raw| {
                raw.readmit()
                    .and_then(|readmitted| {
                        readmitted.positioned().cloned().ok_or_else(|| {
                            "native checkpoint readmission did not produce current authority"
                                .to_string()
                        })
                    })
                    .map_err(|detail| {
                        DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(DurableCheckpointFile {
            checkpoint: DurableCheckpoint {
                coverage: checkpoint.coverage,
                branch_cells: checkpoint.branch_cells,
                branch_roots: checkpoint.branch_roots,
                branch_root_schema_images: checkpoint.branch_root_schema_images,
                record_identity: checkpoint.record_identity,
                record_generation_high_water: checkpoint.record_generation_high_water,
                reusable_record_slots: checkpoint.reusable_record_slots,
                record_slot_frontiers: checkpoint.record_slot_frontiers,
                envelopes,
                partition_images: checkpoint.partition_images,
                aspect_contracts: checkpoint.aspect_contracts,
                lineage: checkpoint.lineage,
                index_definitions: checkpoint.index_definitions,
                derived_index_artifacts: checkpoint.derived_index_artifacts,
                derived_index_checkpoint: checkpoint.derived_index_checkpoint,
                derived_index_checkpoint_format: checkpoint.derived_index_checkpoint_format,
                symbol_table: checkpoint.symbol_table,
                runtime_name: checkpoint.runtime_name,
            },
        })
    }
}
