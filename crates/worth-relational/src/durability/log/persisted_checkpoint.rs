use std::collections::BTreeSet;

use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Serialize};

use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};
use crate::history::data::PositionedCanonicalCommit;

use super::local_store::DurableCheckpointFile;
use super::persisted_canonical_commit::{PersistedCanonicalCommit, PersistedCheckpointCommitRef};

#[cfg(test)]
#[path = "persisted_checkpoint_tests.rs"]
mod tests;

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

struct CheckpointBranchRootRefs<'a> {
    roots: &'a [crate::durability::data::DurableBranchRootImage],
    alias_by_root: &'a [bool],
}

struct CheckpointBranchRootRef<'a> {
    root: &'a crate::durability::data::DurableBranchRootImage,
    alias_shared_partitions: bool,
}

// The alias changes only the native file representation. A recovered root
// still receives its own exact image before digest and schema readmission.
const PARTITION_ALIAS_FORMAT_VERSION: u16 = 1;

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
        // The current storage mirror and an exact branch root can differ after
        // forks or schema evolution. Only identical images may share wire bytes.
        let alias_by_root = branch_roots
            .iter()
            .map(|root| root.partition_images.as_slice() == partition_images.as_slice())
            .collect::<Vec<_>>();
        let aliased_roots = branch_roots
            .iter()
            .zip(&alias_by_root)
            .filter_map(|(root, alias)| alias.then_some(root.commit_id))
            .collect::<Vec<_>>();
        let mut fields = serializer.serialize_struct("PersistedDurableCheckpoint", 20)?;
        fields.serialize_field("coverage", coverage)?;
        fields.serialize_field("branch_cells", branch_cells)?;
        fields.serialize_field(
            "branch_roots",
            &CheckpointBranchRootRefs {
                roots: branch_roots,
                alias_by_root: &alias_by_root,
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
        fields.serialize_field("partition_alias_format", &PARTITION_ALIAS_FORMAT_VERSION)?;
        fields.serialize_field("branch_root_partition_aliases", &aliased_roots)?;
        fields.end()
    }
}

impl Serialize for CheckpointBranchRootRefs<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.roots.len()))?;
        for (root, alias_shared_partitions) in self.roots.iter().zip(self.alias_by_root) {
            sequence.serialize_element(&CheckpointBranchRootRef {
                root,
                alias_shared_partitions: *alias_shared_partitions,
            })?;
        }
        sequence.end()
    }
}

impl Serialize for CheckpointBranchRootRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let crate::durability::data::DurableBranchRootImage {
            format_version,
            commit_id,
            partition_images,
            partition_image_digest,
            schema_carrier_digest,
            root_image_digest,
        } = self.root;
        let mut fields = serializer.serialize_struct("DurableBranchRootImage", 6)?;
        fields.serialize_field("format_version", format_version)?;
        fields.serialize_field("commit_id", commit_id)?;
        fields.serialize_field(
            "partition_images",
            if self.alias_shared_partitions {
                &[][..]
            } else {
                partition_images
            },
        )?;
        fields.serialize_field("partition_image_digest", partition_image_digest)?;
        fields.serialize_field("schema_carrier_digest", schema_carrier_digest)?;
        fields.serialize_field("root_image_digest", root_image_digest)?;
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

fn readmit_partition_aliases(
    checkpoint: &mut PersistedDurableCheckpoint,
) -> Result<(), DurabilityError> {
    let aliases = match checkpoint.partition_alias_format {
        0 if checkpoint.branch_root_partition_aliases.is_empty() => return Ok(()),
        PARTITION_ALIAS_FORMAT_VERSION => &checkpoint.branch_root_partition_aliases,
        _ => {
            return Err(DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "unsupported checkpoint partition alias format",
            ));
        }
    };
    let mut pending = aliases.iter().copied().collect::<BTreeSet<_>>();
    if pending.len() != aliases.len() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "duplicate branch-root partition alias",
        ));
    }
    for root in &mut checkpoint.branch_roots {
        if pending.remove(&root.commit_id) {
            if !root.partition_images.is_empty() {
                return Err(DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    "branch-root partition alias carries an inline image",
                ));
            }
            root.partition_images = checkpoint.partition_images.clone();
        }
    }
    if !pending.is_empty() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "branch-root partition alias names missing root",
        ));
    }
    Ok(())
}
