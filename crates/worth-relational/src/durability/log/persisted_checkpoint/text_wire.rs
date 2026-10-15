//! Borrowed current native checkpoint input; managed text is admitted before ownership.
use super::*;
use serde::Deserialize;
pub(crate) struct CheckpointFileWire<'a> {
    checkpoint: CheckpointWire<'a>,
}
impl<'de: 'a, 'a> serde::Deserialize<'de> for CheckpointFileWire<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct File<'a>(std::marker::PhantomData<&'a str>);
        impl<'de: 'a, 'a> serde::de::Visitor<'de> for File<'a> {
            type Value = CheckpointFileWire<'a>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("current native checkpoint, version first")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                if map.next_key::<&'de str>()? != Some("checkpoint_protocol_version")
                    || map.next_value::<u16>()? != 4
                {
                    return Err(serde::de::Error::custom(
                        "unsupported native checkpoint format; v4 required first",
                    ));
                }
                if map.next_key::<&'de str>()? != Some("checkpoint") {
                    return Err(serde::de::Error::custom("missing current checkpoint"));
                }
                let checkpoint = map.next_value()?;
                if map.next_key::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(serde::de::Error::custom("extra native checkpoint fields"));
                }
                Ok(CheckpointFileWire { checkpoint })
            }
        }
        deserializer.deserialize_map(File(std::marker::PhantomData))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointWire<'a> {
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
    #[serde(borrow)]
    envelopes: Vec<crate::durability::log::persisted_canonical_commit::CanonicalCommitWire<'a>>,
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
    #[serde(borrow)]
    symbol_table: crate::symbols::data::SymbolSnapshotWire<'a>,
    runtime_name: String,
    partition_alias_format: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    branch_root_partition_aliases_v2: Vec<RootPartitionAliases>,
}
impl CheckpointFileWire<'_> {
    pub(crate) fn readmit(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DurableCheckpointFile, crate::durability::data::RelationalRecoveryStop> {
        let input = self.checkpoint;
        super::partition_aliases::require_partition_delta_format(input.partition_alias_format)?;
        let symbol_table = input
            .symbol_table
            .readmit(execution)
            .map_err(crate::symbols::data::field_text_recovery_stop)?;
        let envelopes = input
            .envelopes
            .into_iter()
            .map(|commit| commit.readmit(execution))
            .collect::<Result<Vec<_>, _>>()?;
        PersistedDurableCheckpointFile {
            checkpoint_protocol_version: 4,
            checkpoint: PersistedDurableCheckpoint {
                coverage: input.coverage,
                branch_cells: input.branch_cells,
                branch_roots: input.branch_roots,
                branch_root_schema_images: input.branch_root_schema_images,
                record_identity: input.record_identity,
                record_generation_high_water: input.record_generation_high_water,
                reusable_record_slots: input.reusable_record_slots,
                record_slot_frontiers: input.record_slot_frontiers,
                envelopes: envelopes,
                partition_images: input.partition_images,
                aspect_contracts: input.aspect_contracts,
                lineage: input.lineage,
                index_definitions: input.index_definitions,
                derived_index_artifacts: input.derived_index_artifacts,
                derived_index_checkpoint: input.derived_index_checkpoint,
                derived_index_checkpoint_format: input.derived_index_checkpoint_format,
                symbol_table: symbol_table,
                runtime_name: input.runtime_name,
                partition_alias_format: input.partition_alias_format,
                branch_root_partition_aliases_v2: input.branch_root_partition_aliases_v2,
            },
        }
        .readmit()
        .map_err(Into::into)
    }
}
