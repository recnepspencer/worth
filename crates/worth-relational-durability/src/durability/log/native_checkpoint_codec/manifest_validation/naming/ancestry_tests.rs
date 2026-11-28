//! Linear-time `O(V + E)` ancestry-index validation tests over deep retired
//! chains feeding many surviving leaves, plus cycle, dangling, and parent-head
//! refusal proofs.

use std::collections::BTreeMap;

use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};
use worth_relational_branch::internal_port::branch::registry::checkpoint_capture::data::CheckpointBranchMetadata;
use worth_relational_contracts::internal_port::branch::reference::observation::{
    relational_branch_observation, RelationalBranchReferenceObservation,
};
use worth_relational_contracts::internal_port::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint;
use worth_relational_contracts::internal_port::branch::target::{
    RelationalBranchRootDescriptor, RelationalBranchTarget, RelationalBranchTargetOwnerPort as _,
};
use worth_relational_contracts::internal_port::branch::version::RelationalBranchVersion;
use worth_relational_contracts::internal_port::history::data::coordinates::BranchId;
use worth_relational_contracts::internal_port::lineage::data::artifacts::checkpoint_artifact::LineageCheckpointDigestBasis;
use worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition;
use worth_relational_lineage::internal_port::lineage::naming_pages::{
    BranchNamingRoot, CheckpointNamingState, NamingStateHeader, RetiredBranchFork,
};

use super::ancestry::AncestryIndex;
use crate::durability::data::{
    CheckpointCoverage, CheckpointLineageBasis, RelationalCheckpointManifest,
    RelationalCheckpointManifestDraft,
};
use crate::durability::log::native_checkpoint_codec::member_frame;

fn basis_obs(branch: &str, commit: u64) -> RelationalBranchReferenceObservation {
    let target = RelationalBranchTarget::new(
        1,
        commit,
        commit,
        Vec::new(),
        RelationalBranchRootDescriptor::new([0; 32], [0; 32]),
    );
    relational_branch_observation(
        1,
        branch,
        FoundationalBranchTarget::basis(target),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .unwrap()
}

fn cell(
    branch: BranchId,
    head: u64,
    fork_parent: Option<(BranchId, u64)>,
) -> RelationalBranchCellCheckpoint {
    let (fork_source_branch_id, fork_provenance) = match fork_parent {
        Some((src, fc)) => {
            let obs = basis_obs(&src.0, fc);
            (Some(src), Some(obs))
        }
        None => (None, None),
    };
    RelationalBranchCellCheckpoint {
        runtime_instance_id: 1,
        observation: basis_obs(&branch.0, head),
        branch_id: branch,
        truth_version: RelationalBranchVersion::initial(),
        fork_provenance,
        fork_source_branch_id,
    }
}

fn manifest_with_graph(
    cells: Vec<RelationalBranchCellCheckpoint>,
    retired_names: Vec<BranchId>,
    naming: CheckpointNamingState,
) -> RelationalCheckpointManifest {
    RelationalCheckpointManifestDraft {
        branch_data: CheckpointBranchMetadata::from_ordinary_parts(
            "ancestry-test".into(),
            cells,
            vec![],
            retired_names,
        ),
        coverage: CheckpointCoverage {
            up_to_commit: None,
            up_to_version: None,
            up_to_branch_transition: Default::default(),
        },
        covered_position: PatchStreamPosition(0),
        schemas: Default::default(),
        record_identity: Default::default(),
        aspect_contracts: Vec::new(),
        index_generation_frontier: 1,
        index_definition_frontier: 1,
        index_definitions: Default::default(),
        roots: Default::default(),
        retained_empty_components: Vec::new(),
        mirror_schema: None,
        mirror: Vec::new(),
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
        naming,
        derived_indexes: member_frame::locator(b"derived-indexes"),
        allocations: Default::default(),
    }
    .seal_ordinary()
}

#[test]
fn deep_retired_chain_builds_ancestry_index_linearly_and_enforces_graph_invariants() {
    const CHAIN_LEN: usize = 200;
    const LEAF_COUNT: usize = 200;
    let main = BranchId("main".into());
    let tail = (CHAIN_LEN + LEAF_COUNT + 10) as u64;

    let mut cells = vec![cell(main.clone(), tail, None)];
    let mut retired_names = Vec::with_capacity(CHAIN_LEN);
    let mut retired_forks = Vec::with_capacity(CHAIN_LEN);
    let mut roots = vec![BranchNamingRoot {
        branch: main.clone(),
        naming: Some(NamingStateHeader {
            earliest_event_commit: Some(1),
            latest_event_commit: Some(1),
            unavailable: false,
        }),
        index: None,
        groups: None,
        pages: Vec::new(),
        delta_base: None,
    }];

    let mut prev = main.clone();
    for i in 0..CHAIN_LEN {
        let mid = BranchId(format!("retired-mid-{i:04}"));
        retired_names.push(mid.clone());
        retired_forks.push(RetiredBranchFork {
            branch: mid.clone(),
            parent: prev,
            fork_commit: (i as u64) + 1,
        });
        prev = mid;
    }
    let bottom_retired = prev;
    let mut leaves = Vec::with_capacity(LEAF_COUNT);
    for i in 0..LEAF_COUNT {
        let leaf = BranchId(format!("leaf-{i:04}"));
        cells.push(cell(
            leaf.clone(),
            tail,
            Some((bottom_retired.clone(), (CHAIN_LEN as u64) + 1)),
        ));
        roots.push(BranchNamingRoot {
            branch: leaf.clone(),
            naming: None,
            index: None,
            groups: None,
            pages: Vec::new(),
            delta_base: None,
        });
        leaves.push(leaf);
    }
    roots.sort_by(|a, b| a.branch.cmp(&b.branch));
    retired_forks.sort_by(|a, b| a.branch.cmp(&b.branch));

    let manifest = manifest_with_graph(
        cells.clone(),
        retired_names.clone(),
        CheckpointNamingState {
            branches: roots.clone(),
            group_log: Vec::new(),
            group_log_through: None,
            retired_forks: retired_forks.clone(),
        },
    );
    let index = AncestryIndex::build(&manifest, Some(tail), &BTreeMap::new()).unwrap();
    for leaf in &leaves {
        assert!(index.is_recorded_ancestor(leaf, &main));
        assert!(index.shares_delta_provenance(leaf, &main));
        assert!(index.shares_delta_provenance(leaf, &leaves[0]));
        assert_eq!(
            index.inherited_requirements(leaf, tail),
            (true, false, false)
        );
    }

    // Dangling retired edge (not on any committed branch's ancestor path) is rejected.
    let orphan = BranchId("retired-orphan".into());
    let mut bad_retired_names = retired_names.clone();
    bad_retired_names.push(orphan.clone());
    let mut bad_forks = retired_forks.clone();
    bad_forks.push(RetiredBranchFork {
        branch: orphan,
        parent: main.clone(),
        fork_commit: 1,
    });
    bad_forks.sort_by(|a, b| a.branch.cmp(&b.branch));
    let dangling = manifest_with_graph(
        cells.clone(),
        bad_retired_names,
        CheckpointNamingState {
            branches: roots.clone(),
            group_log: Vec::new(),
            group_log_through: None,
            retired_forks: bad_forks,
        },
    );
    assert!(AncestryIndex::build(&dangling, Some(tail), &BTreeMap::new()).is_err());

    // Combined live + retired cycle (`leaf-0000 -> retired-mid-0199 -> ... -> retired-mid-0000 -> leaf-0000`) is rejected.
    let mut cyclic_forks = retired_forks.clone();
    cyclic_forks[0].parent = leaves[0].clone();
    let cyclic = manifest_with_graph(
        cells,
        retired_names,
        CheckpointNamingState {
            branches: roots,
            group_log: Vec::new(),
            group_log_through: None,
            retired_forks: cyclic_forks,
        },
    );
    assert!(AncestryIndex::build(&cyclic, Some(tail), &BTreeMap::new()).is_err());
}
