//! Linear-time structural validation and fork-forest sealing over deep retired
//! intermediate chains feeding many surviving leaves.

use std::collections::BTreeSet;

use worth_relational_contracts::internal_port::history::data::coordinates::{BranchId, CommitId};
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::CollectedMembers;

use crate::lineage::naming_pages::{BranchNamingRoot, CheckpointNamingState, RetiredBranchFork};
use crate::runtime::state::subsystems::lineage::LineageSubsystem;

#[test]
fn deep_retired_chain_validates_and_builds_fork_forest_in_linear_time() {
    const CHAIN_LEN: usize = 200;
    const LEAF_COUNT: usize = 200;
    let main = BranchId("main".into());

    let mut branches = vec![BranchNamingRoot {
        branch: main.clone(),
        naming: None,
        index: None,
        groups: None,
        pages: Vec::new(),
        delta_base: None,
    }];
    let mut retired_forks = Vec::with_capacity(CHAIN_LEN);
    let mut prev = main.clone();
    for i in 0..CHAIN_LEN {
        let mid = BranchId(format!("retired-mid-{i:04}"));
        retired_forks.push(RetiredBranchFork {
            branch: mid.clone(),
            parent: prev,
            fork_commit: (i as u64) + 1,
        });
        prev = mid;
    }
    let bottom_retired = prev;
    for i in 0..LEAF_COUNT {
        let leaf = BranchId(format!("surviving-leaf-{i:04}"));
        branches.push(BranchNamingRoot {
            branch: leaf,
            naming: None,
            index: None,
            groups: None,
            pages: Vec::new(),
            delta_base: None,
        });
    }
    branches.sort_by(|a, b| a.branch.cmp(&b.branch));
    retired_forks.sort_by(|a, b| a.branch.cmp(&b.branch));

    let state = CheckpointNamingState {
        branches,
        group_log: Vec::new(),
        group_log_through: None,
        retired_forks: retired_forks.clone(),
    };
    assert!(state.validate().is_ok());

    // Closing a cycle at the top of the 200-node retired chain is rejected in O(V + E).
    let mut cyclic = state.clone();
    cyclic.retired_forks[0].parent = bottom_retired.clone();
    assert!(cyclic.validate().is_err());

    // Build a LineageSubsystem with the exact 200-node retired chain feeding 200 leaves
    // and seal through `ForkForest` (`build_fork_forest` + `shares_delta_provenance`).
    let lineage = LineageSubsystem::new();
    let mut committed = vec![main.clone()];
    for edge in &retired_forks {
        lineage.record_fork_ancestry_metadata(
            edge.branch.clone(),
            edge.parent.clone(),
            CommitId(edge.fork_commit),
        );
    }
    for i in 0..LEAF_COUNT {
        let leaf = BranchId(format!("surviving-leaf-{i:04}"));
        lineage.record_fork_ancestry_metadata(
            leaf.clone(),
            bottom_retired.clone(),
            CommitId(CHAIN_LEN as u64 + 1),
        );
        committed.push(leaf);
    }
    let frozen = lineage.freeze_naming();
    let mut sink = CollectedMembers::default();
    let sealed = frozen.seal(&committed, &mut sink).unwrap();
    assert_eq!(sealed.branches.len(), LEAF_COUNT + 1);
    assert_eq!(sealed.retired_forks.len(), CHAIN_LEN);
    let emitted: BTreeSet<&BranchId> = sealed.retired_forks.iter().map(|e| &e.branch).collect();
    assert_eq!(emitted.len(), CHAIN_LEN);
    assert!(sealed.validate().is_ok());
}
