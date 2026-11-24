//! Bare selected region directory for head/history transport courts.
use super::{
    CheckpointCoverage, CheckpointLineageBasis, RelationalCheckpointManifest,
    RelationalCheckpointManifestDraft,
};
use crate::branch::registry::CheckpointBranchMetadata;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::lineage::data::LineageCheckpointDigestBasis;
use crate::storage::data::checkpoint_region::RelationalCheckpointRegion;

pub(crate) fn selected_region_manifest(
    leaf: RelationalCheckpointRegion,
    schema_digest: [u8; 32],
    derived_indexes: RelationalDurableMemberLocator,
) -> RelationalCheckpointManifest {
    RelationalCheckpointManifestDraft {
        branch_data: CheckpointBranchMetadata::from_ordinary_parts(
            "selected-region-custody".into(),
            vec![],
            vec![],
            vec![],
        ),
        coverage: CheckpointCoverage {
            up_to_commit: None,
            up_to_version: None,
            up_to_branch_transition: Default::default(),
        },
        covered_position: crate::publication::patch::data::PatchStreamPosition(0),
        schemas: Default::default(),
        record_identity: Default::default(),
        aspect_contracts: Vec::new(),
        index_generation_frontier: 0,
        index_definition_frontier: 0,
        index_definitions: Default::default(),
        roots: Default::default(),
        retained_empty_components: Vec::new(),
        mirror_schema: Some(schema_digest),
        mirror: vec![leaf],
        history_dependencies: Vec::new(),
        commits: Default::default(),
        canonical_members: Default::default(),
        symbols: Default::default(),
        symbol_identity_frontier: 0,
        symbol_buckets: Default::default(),
        lineage: Default::default(),
        lineage_events: Default::default(),
        lineage_basis: CheckpointLineageBasis::from_ordinary(LineageCheckpointDigestBasis::new(
            0,
            Vec::new(),
            0,
            0,
        )),
        derived_indexes,
        allocations: Default::default(),
    }
    .seal_ordinary()
}
