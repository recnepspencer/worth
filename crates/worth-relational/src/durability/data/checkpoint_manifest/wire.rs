//! Exact original flat manifest grammar, including compact struct order.
use super::*;

#[derive(Serialize)]
#[serde(rename = "RelationalCheckpointManifest")]
struct ManifestView<'a> {
    coverage: &'a CheckpointCoverage,
    covered_position: &'a PatchStreamPosition,
    runtime_name: &'a str,
    branch_cells: &'a [crate::branch::RelationalBranchCellCheckpoint],
    branch_lifecycle: &'a [DurableBranchLifecycleImage],
    retired_branch_names: &'a [crate::history::data::BranchId],
    schemas: &'a super::CheckpointSchemaImages,
    record_identity: &'a DurableRecordIdentityState,
    aspect_contracts: &'a Vec<worth_foundational::facade::PortableAspectContract>,
    index_generation_frontier: &'a u64,
    index_definition_frontier: &'a u64,
    index_definitions: &'a super::CheckpointIndexDefinitions,
    roots: &'a super::CheckpointRoots,
    retained_empty_components: &'a Vec<RelationalCheckpointEmptyComponent>,
    mirror_schema: &'a Option<[u8; 32]>,
    mirror: &'a Vec<RelationalCheckpointRegion>,
    history_dependencies: &'a Vec<RelationalCheckpointHistoryDependency>,
    commits: &'a super::CheckpointCommitIndex,
    canonical_members:
        &'a crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers,
    symbols: &'a super::CheckpointPages,
    symbol_identity_frontier: &'a u32,
    symbol_buckets: &'a super::CheckpointSymbolBuckets,
    lineage: &'a super::CheckpointPages,
    lineage_events: &'a super::CheckpointLineageEventPages,
    lineage_basis: &'a super::CheckpointLineageBasis,
    derived_indexes: &'a RelationalDurableMemberLocator,
}

#[derive(Deserialize)]
#[serde(rename = "RelationalCheckpointManifest", deny_unknown_fields)]
struct ManifestWire {
    coverage: CheckpointCoverage,
    covered_position: PatchStreamPosition,
    runtime_name: String,
    branch_cells: Vec<crate::branch::RelationalBranchCellCheckpoint>,
    branch_lifecycle: Vec<DurableBranchLifecycleImage>,
    retired_branch_names: Vec<crate::history::data::BranchId>,
    schemas: Vec<DurableBranchRootSchemaImage>,
    record_identity: DurableRecordIdentityState,
    aspect_contracts: Vec<worth_foundational::facade::PortableAspectContract>,
    index_generation_frontier: u64,
    index_definition_frontier: u64,
    index_definitions: Vec<crate::indexes::data::DerivedIndexDefinition>,
    roots: super::CheckpointRoots,
    retained_empty_components: Vec<RelationalCheckpointEmptyComponent>,
    mirror_schema: Option<[u8; 32]>,
    mirror: Vec<RelationalCheckpointRegion>,
    history_dependencies: Vec<RelationalCheckpointHistoryDependency>,
    commits: super::CheckpointCommitIndex,
    canonical_members: crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers,
    symbols: Vec<RelationalCheckpointPage>,
    symbol_identity_frontier: u32,
    symbol_buckets: Vec<RelationalCheckpointSymbolBucket>,
    lineage: Vec<RelationalCheckpointPage>,
    lineage_events: Vec<RelationalCheckpointLineageEventPage>,
    lineage_basis: crate::lineage::data::LineageCheckpointDigestBasis,
    derived_indexes: RelationalDurableMemberLocator,
}

impl Serialize for RelationalCheckpointManifestDraft {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ManifestView {
            coverage: &self.coverage,
            covered_position: &self.covered_position,
            runtime_name: &self.runtime_name,
            branch_cells: &self.branch_cells,
            branch_lifecycle: &self.branch_lifecycle,
            retired_branch_names: &self.retired_branch_names,
            schemas: &self.schemas,
            record_identity: &self.record_identity,
            aspect_contracts: &self.aspect_contracts,
            index_generation_frontier: &self.index_generation_frontier,
            index_definition_frontier: &self.index_definition_frontier,
            index_definitions: &self.index_definitions,
            roots: &self.roots,
            retained_empty_components: &self.retained_empty_components,
            mirror_schema: &self.mirror_schema,
            mirror: &self.mirror,
            history_dependencies: &self.history_dependencies,
            commits: &self.commits,
            canonical_members: &self.canonical_members,
            symbols: &self.symbols,
            symbol_identity_frontier: &self.symbol_identity_frontier,
            symbol_buckets: &self.symbol_buckets,
            lineage: &self.lineage,
            lineage_events: &self.lineage_events,
            lineage_basis: &self.lineage_basis,
            derived_indexes: &self.derived_indexes,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RelationalCheckpointManifestDraft {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = ManifestWire::deserialize(deserializer)?;
        Ok(RelationalCheckpointManifestDraft {
            branch_data: crate::branch::CheckpointBranchMetadata::from_ordinary_parts(
                wire.runtime_name,
                wire.branch_cells,
                wire.branch_lifecycle,
                wire.retired_branch_names,
            ),
            coverage: wire.coverage,
            covered_position: wire.covered_position,
            schemas: super::CheckpointSchemaImages::from_ordinary(wire.schemas),
            record_identity: wire.record_identity,
            aspect_contracts: wire.aspect_contracts,
            index_generation_frontier: wire.index_generation_frontier,
            index_definition_frontier: wire.index_definition_frontier,
            index_definitions: super::CheckpointIndexDefinitions::from_ordinary(
                wire.index_definitions,
            ),
            roots: wire.roots,
            retained_empty_components: wire.retained_empty_components,
            mirror_schema: wire.mirror_schema,
            mirror: wire.mirror,
            history_dependencies: wire.history_dependencies,
            commits: wire.commits,
            canonical_members: wire.canonical_members,
            symbols: super::CheckpointPages::from_ordinary(wire.symbols),
            symbol_identity_frontier: wire.symbol_identity_frontier,
            symbol_buckets: super::CheckpointSymbolBuckets::from_ordinary(wire.symbol_buckets),
            lineage: super::CheckpointPages::from_ordinary(wire.lineage),
            lineage_events: super::CheckpointLineageEventPages::from_ordinary(wire.lineage_events),
            lineage_basis: super::CheckpointLineageBasis::from_ordinary(wire.lineage_basis),
            derived_indexes: wire.derived_indexes,
            allocations: Default::default(),
        })
    }
}
impl<'de> Deserialize<'de> for RelationalCheckpointManifest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        RelationalCheckpointManifestDraft::deserialize(deserializer)
            .map(|draft| draft.seal_ordinary())
    }
}

// Independent prior derive grammar: the original flat struct's field sequence,
// without using ManifestView or projecting through public DurableCheckpoint.
#[cfg(test)]
struct OriginalFlat<'a>(&'a RelationalCheckpointManifest);
#[cfg(test)]
impl Serialize for OriginalFlat<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let manifest = self.0;
        let mut state = serializer.serialize_struct("RelationalCheckpointManifest", 26)?;
        state.serialize_field("coverage", &manifest.coverage)?;
        state.serialize_field("covered_position", &manifest.covered_position)?;
        state.serialize_field("runtime_name", &manifest.runtime_name)?;
        state.serialize_field("branch_cells", &manifest.branch_cells)?;
        state.serialize_field("branch_lifecycle", &manifest.branch_lifecycle)?;
        state.serialize_field("retired_branch_names", &manifest.retired_branch_names)?;
        state.serialize_field("schemas", &manifest.schemas)?;
        state.serialize_field("record_identity", &manifest.record_identity)?;
        state.serialize_field("aspect_contracts", &manifest.aspect_contracts)?;
        state.serialize_field(
            "index_generation_frontier",
            &manifest.index_generation_frontier,
        )?;
        state.serialize_field(
            "index_definition_frontier",
            &manifest.index_definition_frontier,
        )?;
        state.serialize_field("index_definitions", &manifest.index_definitions)?;
        state.serialize_field("roots", &manifest.roots)?;
        state.serialize_field(
            "retained_empty_components",
            &manifest.retained_empty_components,
        )?;
        state.serialize_field("mirror_schema", &manifest.mirror_schema)?;
        state.serialize_field("mirror", &manifest.mirror)?;
        state.serialize_field("history_dependencies", &manifest.history_dependencies)?;
        state.serialize_field("commits", &manifest.commits)?;
        state.serialize_field("canonical_members", &manifest.canonical_members)?;
        state.serialize_field("symbols", &manifest.symbols)?;
        state.serialize_field(
            "symbol_identity_frontier",
            &manifest.symbol_identity_frontier,
        )?;
        state.serialize_field("symbol_buckets", &manifest.symbol_buckets)?;
        state.serialize_field("lineage", &manifest.lineage)?;
        state.serialize_field("lineage_events", &manifest.lineage_events)?;
        state.serialize_field("lineage_basis", &manifest.lineage_basis)?;
        state.serialize_field("derived_indexes", &manifest.derived_indexes)?;
        state.end()
    }
}

impl RelationalCheckpointManifest {
    /// Borrowed assertion called only by the genuine manifest codec fixture.
    #[cfg(test)]
    pub(crate) fn assert_original_flat_wire(&self) {
        let old_named = rmp_serde::to_vec_named(&OriginalFlat(self)).unwrap();
        let old_compact = rmp_serde::to_vec(&OriginalFlat(self)).unwrap();
        assert_eq!(rmp_serde::to_vec_named(self).unwrap(), old_named);
        assert_eq!(rmp_serde::to_vec(self).unwrap(), old_compact);
        for bytes in [old_named, old_compact] {
            let decoded: RelationalCheckpointManifest = rmp_serde::from_slice(&bytes).unwrap();
            assert_eq!(&decoded, self);
        }
    }
}

impl Serialize for RelationalCheckpointManifest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.data.serialize(serializer)
    }
}
