use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Serialize};

use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};
use crate::history::data::PositionedCanonicalCommit;

use super::local_store::DurableCheckpointFile;
use super::persisted_canonical_commit::{PersistedCanonicalCommit, PersistedCheckpointCommitRef};

#[cfg(test)]
mod image_refusal_tests;
mod native_format;
mod partition_aliases;
mod section_accounting;
#[cfg(test)]
#[path = "persisted_checkpoint_tests.rs"]
mod tests;

pub(super) use native_format::undecodable_checkpoint;
use native_format::{readmit_native_checkpoint_format, NATIVE_CHECKPOINT_FORMAT_VERSION};
use partition_aliases::{
    readmit_partition_aliases, CheckpointBranchRootRefs, PartitionAliasPlan, RootPartitionAliases,
    PARTITION_DELTA_FORMAT_VERSION,
};
pub(super) use section_accounting::CaptureSectionRecorder;
use section_accounting::{MeasuredValue, NativeSection};

#[derive(Serialize, Deserialize)]
pub(super) struct PersistedDurableCheckpointFile {
    checkpoint: PersistedDurableCheckpoint,
}

/// Borrowed checkpoint-file encoder. Only the output byte vector is large;
/// the checkpoint image and its canonical envelopes stay in one owner.
pub(super) struct PersistedDurableCheckpointFileRef<'a> {
    checkpoint: &'a DurableCheckpoint,
    recorder: Option<&'a CaptureSectionRecorder>,
}

struct PersistedDurableCheckpointRef<'a> {
    checkpoint: &'a DurableCheckpoint,
    recorder: Option<&'a CaptureSectionRecorder>,
}

struct CheckpointEnvelopeRefs<'a>(&'a [PositionedCanonicalCommit]);

impl<'a> PersistedDurableCheckpointFileRef<'a> {
    pub(super) fn new(checkpoint: &'a DurableCheckpoint) -> Self {
        Self {
            checkpoint,
            recorder: None,
        }
    }

    pub(super) fn measured(
        checkpoint: &'a DurableCheckpoint,
        recorder: &'a CaptureSectionRecorder,
    ) -> Self {
        Self {
            checkpoint,
            recorder: Some(recorder),
        }
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
            &PersistedDurableCheckpointRef {
                checkpoint: self.checkpoint,
                recorder: self.recorder,
            },
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
            retired_branch_names,
            branch_roots,
            branch_root_schema_images,
            record_identity,
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
        } = self.checkpoint;
        // Exact shared partitions are omitted on the wire, while divergent
        // partitions retain their independently readmitted image.
        let aliases = PartitionAliasPlan::for_checkpoint(self.checkpoint)
            .map_err(serde::ser::Error::custom)?;
        let mut fields = serializer.serialize_struct("PersistedDurableCheckpoint", 19)?;
        fields.serialize_field("native_format", &NATIVE_CHECKPOINT_FORMAT_VERSION)?;
        fields.serialize_field("coverage", coverage)?;
        fields.serialize_field(
            "branch_cells",
            &MeasuredValue {
                value: branch_cells,
                section: NativeSection::BranchCells,
                recorder: self.recorder,
            },
        )?;
        fields.serialize_field(
            "retired_branch_names",
            &MeasuredValue {
                value: retired_branch_names,
                section: NativeSection::BranchCells,
                recorder: self.recorder,
            },
        )?;
        let root_refs = CheckpointBranchRootRefs {
            roots: branch_roots,
            aliases: &aliases.roots,
        };
        fields.serialize_field(
            "branch_roots",
            &MeasuredValue {
                value: &root_refs,
                section: NativeSection::BranchRoots,
                recorder: self.recorder,
            },
        )?;
        fields.serialize_field(
            "branch_root_schema_images",
            &MeasuredValue {
                value: branch_root_schema_images,
                section: NativeSection::BranchRoots,
                recorder: self.recorder,
            },
        )?;
        fields.serialize_field("record_identity", record_identity)?;
        fields.serialize_field(
            "envelopes",
            &MeasuredValue {
                value: &CheckpointEnvelopeRefs(envelopes),
                section: NativeSection::Envelopes,
                recorder: self.recorder,
            },
        )?;
        fields.serialize_field(
            "partition_images",
            &MeasuredValue {
                value: partition_images,
                section: NativeSection::PartitionMirror,
                recorder: self.recorder,
            },
        )?;
        fields.serialize_field("aspect_contracts", aspect_contracts)?;
        fields.serialize_field("lineage", lineage)?;
        fields.serialize_field(
            "index_definitions",
            &MeasuredValue {
                value: index_definitions,
                section: NativeSection::DerivedIndexes,
                recorder: self.recorder,
            },
        )?;
        fields.serialize_field(
            "derived_index_artifacts",
            &MeasuredValue {
                value: derived_index_artifacts,
                section: NativeSection::DerivedIndexes,
                recorder: self.recorder,
            },
        )?;
        fields.serialize_field(
            "derived_index_checkpoint",
            &MeasuredValue {
                value: derived_index_checkpoint,
                section: NativeSection::DerivedIndexes,
                recorder: self.recorder,
            },
        )?;
        fields.serialize_field(
            "derived_index_checkpoint_format",
            derived_index_checkpoint_format,
        )?;
        fields.serialize_field("symbol_table", symbol_table)?;
        fields.serialize_field("runtime_name", runtime_name)?;
        fields.serialize_field("partition_alias_format", &PARTITION_DELTA_FORMAT_VERSION)?;
        fields.serialize_field(
            "branch_root_partition_aliases_v2",
            &MeasuredValue {
                value: &aliases.roots,
                section: NativeSection::BranchRoots,
                recorder: self.recorder,
            },
        )?;
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

/// The checkpoint wire image. This build writes every field, so an image
/// missing one is refused at decode; only `native_format` may be absent, which
/// is how an image from before the format field is recognized and refused.
#[derive(Serialize, Deserialize)]
struct PersistedDurableCheckpoint {
    #[serde(default)]
    native_format: u16,
    coverage: crate::durability::data::CheckpointCoverage,
    branch_cells: Vec<crate::branch::RelationalBranchCellCheckpoint>,
    retired_branch_names: Vec<crate::history::data::BranchId>,
    branch_roots: Vec<crate::durability::data::DurableBranchRootImage>,
    branch_root_schema_images: Vec<crate::durability::data::DurableBranchRootSchemaImage>,
    record_identity: crate::durability::data::DurableRecordIdentityState,
    envelopes: Vec<PersistedCanonicalCommit>,
    partition_images: Vec<crate::durability::data::PartitionCheckpointImage>,
    aspect_contracts: Vec<worth_foundational::facade::PortableAspectContract>,
    lineage: crate::lineage::data::LineageCheckpointArtifact,
    index_definitions: Vec<crate::indexes::data::DerivedIndexDefinition>,
    derived_index_artifacts: crate::indexes::data::DerivedIndexArtifacts,
    #[serde(deserialize_with = "Option::deserialize")]
    derived_index_checkpoint:
        Option<crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>,
    derived_index_checkpoint_format: u16,
    symbol_table: crate::symbols::data::SymbolTableSnapshot,
    runtime_name: String,
    partition_alias_format: u16,
    branch_root_partition_aliases_v2: Vec<RootPartitionAliases>,
}

impl PersistedDurableCheckpointFile {
    pub(super) fn readmit(self) -> Result<DurableCheckpointFile, DurabilityError> {
        let mut checkpoint = self.checkpoint;
        readmit_native_checkpoint_format(checkpoint.native_format)?;
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
                retired_branch_names: checkpoint.retired_branch_names,
                branch_roots: checkpoint.branch_roots,
                branch_root_schema_images: checkpoint.branch_root_schema_images,
                record_identity: checkpoint.record_identity,
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
