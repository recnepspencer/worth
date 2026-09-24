use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Serialize};

use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};
use crate::history::data::PositionedCanonicalCommit;

use super::local_store::DurableCheckpointFile;
use super::persisted_canonical_commit::{PersistedCanonicalCommit, PersistedCheckpointCommitRef};

mod partition_aliases;
#[cfg(test)]
#[path = "persisted_checkpoint_tests.rs"]
mod tests;

use partition_aliases::{
    readmit_partition_aliases, CheckpointBranchRootRefs, PartitionAliasPlan, RootPartitionAliases,
    PARTITION_DELTA_FORMAT_VERSION,
};

#[derive(Serialize, Deserialize)]
pub(super) struct PersistedDurableCheckpointFile {
    checkpoint: PersistedDurableCheckpoint,
}

/// Borrowed checkpoint-file encoder. Only the output byte vector is large;
/// the checkpoint image and its canonical envelopes stay in one owner.
pub(super) struct PersistedDurableCheckpointFileRef<'a> {
    checkpoint: &'a DurableCheckpoint,
}

struct PersistedDurableCheckpointRef<'a>(&'a DurableCheckpoint);

struct CheckpointEnvelopeRefs<'a>(&'a [PositionedCanonicalCommit]);

impl<'a> PersistedDurableCheckpointFileRef<'a> {
    pub(super) fn new(checkpoint: &'a DurableCheckpoint) -> Self {
        Self { checkpoint }
    }
}

impl Serialize for PersistedDurableCheckpointFileRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut fields = serializer.serialize_struct("PersistedDurableCheckpointFile", 1)?;
        fields.serialize_field(
            "checkpoint",
            &PersistedDurableCheckpointRef(self.checkpoint),
        )?;
        fields.end()
    }
}

impl Serialize for PersistedDurableCheckpointRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
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
        } = self.0;
        // Exact shared partitions are omitted on the wire, while divergent
        // partitions retain their independently readmitted image.
        let aliases =
            PartitionAliasPlan::for_checkpoint(self.0).map_err(serde::ser::Error::custom)?;
        let mut fields = serializer.serialize_struct("PersistedDurableCheckpoint", 21)?;
        fields.serialize_field("coverage", coverage)?;
        fields.serialize_field("branch_cells", branch_cells)?;
        fields.serialize_field(
            "branch_roots",
            &CheckpointBranchRootRefs {
                roots: branch_roots,
                aliases: &aliases.roots,
            },
        )?;
        fields.serialize_field("branch_root_schema_images", branch_root_schema_images)?;
        fields.serialize_field("record_identity", record_identity)?;
        fields.serialize_field("record_generation_high_water", record_generation_high_water)?;
        fields.serialize_field("reusable_record_slots", reusable_record_slots)?;
        fields.serialize_field("record_slot_frontiers", record_slot_frontiers)?;
        fields.serialize_field("envelopes", &CheckpointEnvelopeRefs(envelopes))?;
        fields.serialize_field("partition_images", partition_images)?;
        fields.serialize_field("aspect_contracts", aspect_contracts)?;
        fields.serialize_field("lineage", lineage)?;
        fields.serialize_field("index_definitions", index_definitions)?;
        fields.serialize_field("derived_index_artifacts", derived_index_artifacts)?;
        fields.serialize_field("derived_index_checkpoint", derived_index_checkpoint)?;
        fields.serialize_field(
            "derived_index_checkpoint_format",
            derived_index_checkpoint_format,
        )?;
        fields.serialize_field("symbol_table", symbol_table)?;
        fields.serialize_field("runtime_name", runtime_name)?;
        fields.serialize_field("partition_alias_format", &PARTITION_DELTA_FORMAT_VERSION)?;
        fields.serialize_field(
            "branch_root_partition_aliases",
            &[] as &[crate::history::data::CommitId],
        )?;
        fields.serialize_field("branch_root_partition_aliases_v2", &aliases.roots)?;
        fields.end()
    }
}

impl Serialize for CheckpointEnvelopeRefs<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for envelope in self.0 {
            sequence.serialize_element(&PersistedCheckpointCommitRef::from_positioned(envelope))?;
        }
        sequence.end()
    }
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
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    partition_alias_format: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    branch_root_partition_aliases: Vec<crate::history::data::CommitId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    branch_root_partition_aliases_v2: Vec<RootPartitionAliases>,
}

fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}

impl PersistedDurableCheckpointFile {
    #[cfg(test)]
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
                partition_alias_format: 0,
                branch_root_partition_aliases: Vec::new(),
                branch_root_partition_aliases_v2: Vec::new(),
            },
        }
    }

    pub(super) fn readmit(self) -> Result<DurableCheckpointFile, DurabilityError> {
        let mut checkpoint = self.checkpoint;
        readmit_partition_aliases(&mut checkpoint)?;
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
