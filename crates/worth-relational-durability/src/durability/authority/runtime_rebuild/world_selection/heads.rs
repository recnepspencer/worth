//! Owning-branch linear-predecessor resolution for co-existing live World heads
//! sharing a Native branch.
use super::*;
use contracts::history::data::coordinates::{BranchId, CommitId};
use std::collections::{BTreeMap, BTreeSet};

struct ResolvedLiveHead<'a> {
    source: &'a Source,
    root: Arc<RelationalBranchRoot>,
    descriptor: RelationalBranchBasisDescriptor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PrimaryCommitEdge {
    authoring_branch: BranchId,
    primary_parent: Option<CommitId>,
}

impl WorldSelectionReplay {
    /// Validate every live World head and select the unique ancestry tip per
    /// Native branch so co-existing `ReuseExact` World branches sharing one
    /// Native branch retain their exact ancestor roots without displacing a
    /// newer `Published` head, while any divergent, unproven Empty,
    /// side-parent-only, missing-link, cyclic, or stale non-ancestor head
    /// fails closed with `CorruptCheckpoint`.
    pub(super) fn select_live_heads(
        &self,
        request: &ExecutionRequest<'_, '_>,
        inputs: &WorldReplayInputs<'_>,
        plan: &RecoveryPlan,
        roots: &mut RecoveredRootInventory,
    ) -> Result<(), Stop> {
        let mut by_branch: BTreeMap<BranchId, Vec<ResolvedLiveHead<'_>>> = BTreeMap::new();
        for source in self.admitted.source().heads() {
            let recorded = self.recorded_source(source, inputs, plan, roots, request)?;
            let (root, descriptor) = roots
                .exact_image(
                    request,
                    inputs.history,
                    inputs.registry,
                    inputs.symbols,
                    &recorded,
                )?
                .ok_or_else(|| corrupt("World selected source image is unavailable"))?;
            self.require_source(&recorded, &root, inputs, plan, request)?;
            by_branch
                .entry(descriptor.branch_id().clone())
                .or_default()
                .push(ResolvedLiveHead {
                    source,
                    root,
                    descriptor,
                });
        }

        for (branch, group) in &by_branch {
            let lookup = |commit: CommitId| {
                inputs
                    .history
                    .commit_artifact(commit)
                    .map(|artifact| PrimaryCommitEdge {
                        authoring_branch: artifact.identity().authoring_branch().clone(),
                        primary_parent: artifact.parentage().as_slice().first().copied(),
                    })
            };
            let consult = || {
                request
                    .consult()
                    .map_err(|cause| Stop::Request(cause.into()))
            };
            let tip_idx = select_unique_tip_index(
                branch,
                group,
                same_head,
                |head| head.root.commit_id(),
                &lookup,
                &consult,
            )?;
            self.select(request, inputs, plan, roots, group[tip_idx].source, true)?;
        }
        Ok(())
    }
}

/// Production tip selector shared across live replay and unit tests: verifies
/// strict asymmetric reachability (`older < newer` and NOT `newer < older`)
/// along `branch`'s own linear primary-predecessor chain, failing closed with
/// `CorruptCheckpoint` on mutual reachability (cycles), divergence, unproven
/// Empty heads, merge side-parent-only reachability, or missing history links.
fn select_unique_tip_index<T>(
    branch: &BranchId,
    group: &[T],
    same: impl Fn(&T, &T) -> bool,
    commit_of: impl Fn(&T) -> Option<CommitId>,
    lookup: &impl Fn(CommitId) -> Option<PrimaryCommitEdge>,
    consult: &impl Fn() -> Result<(), Stop>,
) -> Result<usize, Stop> {
    let mut tip = 0usize;
    for idx in 1..group.len() {
        if same(&group[idx], &group[tip]) {
            continue;
        }
        let tip_before_idx = is_owning_branch_linear_ancestor(
            branch,
            commit_of(&group[tip]),
            commit_of(&group[idx]),
            lookup,
            consult,
        )?;
        let idx_before_tip = is_owning_branch_linear_ancestor(
            branch,
            commit_of(&group[idx]),
            commit_of(&group[tip]),
            lookup,
            consult,
        )?;
        match (tip_before_idx, idx_before_tip) {
            (true, false) => tip = idx,
            (false, true) => {}
            (true, true) | (false, false) => {
                return Err(corrupt(
                    "World heads on a Native branch do not share one acyclic linear owning-branch history",
                ));
            }
        }
    }
    for (idx, head) in group.iter().enumerate() {
        if idx == tip || same(head, &group[tip]) {
            continue;
        }
        let head_before_tip = is_owning_branch_linear_ancestor(
            branch,
            commit_of(head),
            commit_of(&group[tip]),
            lookup,
            consult,
        )?;
        let tip_before_head = is_owning_branch_linear_ancestor(
            branch,
            commit_of(&group[tip]),
            commit_of(head),
            lookup,
            consult,
        )?;
        if !head_before_tip || tip_before_head {
            return Err(corrupt(
                "World heads on a Native branch do not share one acyclic linear owning-branch history",
            ));
        }
    }
    Ok(tip)
}

fn same_head(a: &ResolvedLiveHead<'_>, b: &ResolvedLiveHead<'_>) -> bool {
    a.root.commit_id() == b.root.commit_id()
        && a.descriptor == b.descriptor
        && Arc::ptr_eq(&a.root, &b.root)
}

/// Proves that `older` strictly precedes `newer` along `branch`'s own linear
/// primary-predecessor chain:
/// - Walks ONLY `parents[0]` (never merge side-parents `parents[1..]`).
/// - Requires every intermediate commit along the chain to exist in `history`
///   and be authored on `branch` (a commit authored on a foreign branch can be
///   the exact fork-base `older` reached via a primary edge from `branch`, but
///   is never walked past into foreign branch history).
/// - Accepts `older = None` (Empty) ONLY when the complete primary chain from
///   `newer` terminates without missing links at a root commit authored on
///   `branch` whose primary parent is `None`.
fn is_owning_branch_linear_ancestor(
    branch: &BranchId,
    older: Option<CommitId>,
    newer: Option<CommitId>,
    lookup: &impl Fn(CommitId) -> Option<PrimaryCommitEdge>,
    consult: &impl Fn() -> Result<(), Stop>,
) -> Result<bool, Stop> {
    if older == newer {
        return Ok(false);
    }
    let Some(mut current) = newer else {
        return Ok(false);
    };
    let mut visited: BTreeSet<CommitId> = BTreeSet::new();
    loop {
        consult()?;
        if !visited.insert(current) {
            return Ok(false);
        }
        let Some(edge) = lookup(current) else {
            return Ok(false);
        };
        if &edge.authoring_branch != branch {
            return Ok(false);
        }
        match (older, edge.primary_parent) {
            (Some(target), Some(parent)) if parent == target => return Ok(true),
            (Some(_), Some(parent)) => current = parent,
            (Some(_), None) => return Ok(false),
            (None, None) => return Ok(true),
            (None, Some(parent)) => current = parent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contracts::durability::data::recovery_errors::RecoveryFailureClass;

    fn run_selector(
        branch: &BranchId,
        heads: &[Option<CommitId>],
        edges: &BTreeMap<CommitId, PrimaryCommitEdge>,
    ) -> Result<usize, Stop> {
        let lookup = |id: CommitId| edges.get(&id).cloned();
        let consult = || Ok(());
        select_unique_tip_index(branch, heads, |a, b| a == b, |h| *h, &lookup, &consult)
    }

    fn assert_corrupt_checkpoint(res: Result<usize, Stop>, msg: &str) {
        match res {
            Err(Stop::Durability(err)) if err.class == RecoveryFailureClass::CorruptCheckpoint => {}
            other => panic!("{msg}: expected CorruptCheckpoint, got {other:?}"),
        }
    }

    fn edge(branch: &str, primary_parent: Option<u64>) -> PrimaryCommitEdge {
        PrimaryCommitEdge {
            authoring_branch: BranchId(branch.to_owned()),
            primary_parent: primary_parent.map(CommitId),
        }
    }

    #[test]
    fn linear_owning_branch_chain_from_empty_selects_tip() {
        let main = BranchId("main".to_owned());
        let edges = BTreeMap::from([
            (CommitId(1), edge("main", None)),
            (CommitId(2), edge("main", Some(1))),
            (CommitId(3), edge("main", Some(2))),
        ]);
        let tip = run_selector(
            &main,
            &[
                None,
                Some(CommitId(1)),
                Some(CommitId(3)),
                Some(CommitId(2)),
            ],
            &edges,
        )
        .expect("linear chain from Empty must select tip");
        assert_eq!(tip, 2);
    }

    #[test]
    fn fork_base_primary_predecessor_is_accepted_but_pre_fork_foreign_ancestor_is_rejected() {
        let feat = BranchId("feat".to_owned());
        let edges = BTreeMap::from([
            (CommitId(1), edge("main", None)),
            (CommitId(2), edge("main", Some(1))),
            (CommitId(3), edge("feat", Some(2))),
            (CommitId(4), edge("feat", Some(3))),
        ]);
        let tip = run_selector(&feat, &[Some(CommitId(2)), Some(CommitId(4))], &edges)
            .expect("exact fork base on owning primary chain must be accepted");
        assert_eq!(tip, 1);
        assert_corrupt_checkpoint(
            run_selector(&feat, &[Some(CommitId(1)), Some(CommitId(4))], &edges),
            "pre-fork ancestor on foreign branch must fail closed",
        );
    }

    #[test]
    fn rejects_unproven_empty_when_branch_forked_from_non_empty_or_root_missing() {
        let feat = BranchId("feat".to_owned());
        let forked_edges = BTreeMap::from([
            (CommitId(1), edge("main", None)),
            (CommitId(2), edge("feat", Some(1))),
        ]);
        assert_corrupt_checkpoint(
            run_selector(&feat, &[None, Some(CommitId(2))], &forked_edges),
            "Empty on a branch forked from a non-empty foreign commit must fail closed",
        );

        let main = BranchId("main".to_owned());
        let pruned_root_edges = BTreeMap::from([(CommitId(2), edge("main", Some(1)))]);
        assert_corrupt_checkpoint(
            run_selector(&main, &[None, Some(CommitId(2))], &pruned_root_edges),
            "Empty without a complete history edge to an Empty-parent root must fail closed",
        );
    }

    #[test]
    fn rejects_merge_side_parent_only_reachability() {
        let main = BranchId("main".to_owned());
        let edges = BTreeMap::from([
            (CommitId(1), edge("main", None)),
            (CommitId(2), edge("main", Some(1))),
            (CommitId(10), edge("side", Some(1))),
            (CommitId(3), edge("main", Some(2))),
        ]);
        assert_corrupt_checkpoint(
            run_selector(&main, &[Some(CommitId(10)), Some(CommitId(3))], &edges),
            "merge side-parent-only reachability must fail closed",
        );
    }

    #[test]
    fn rejects_missing_intermediate_link_and_divergent_or_cyclic_heads() {
        let main = BranchId("main".to_owned());
        let missing_link = BTreeMap::from([
            (CommitId(1), edge("main", None)),
            (CommitId(3), edge("main", Some(2))),
        ]);
        assert_corrupt_checkpoint(
            run_selector(
                &main,
                &[Some(CommitId(1)), Some(CommitId(3))],
                &missing_link,
            ),
            "missing intermediate commit link must fail closed",
        );

        let divergent = BTreeMap::from([
            (CommitId(1), edge("main", None)),
            (CommitId(2), edge("main", Some(1))),
            (CommitId(3), edge("main", Some(1))),
        ]);
        assert_corrupt_checkpoint(
            run_selector(&main, &[Some(CommitId(2)), Some(CommitId(3))], &divergent),
            "divergent heads must fail closed",
        );

        let cyclic = BTreeMap::from([
            (CommitId(1), edge("main", Some(2))),
            (CommitId(2), edge("main", Some(1))),
        ]);
        assert_corrupt_checkpoint(
            run_selector(&main, &[None, Some(CommitId(2))], &cyclic),
            "cyclic parentage with Empty must fail closed",
        );
        assert_corrupt_checkpoint(
            run_selector(&main, &[Some(CommitId(1)), Some(CommitId(2))], &cyclic),
            "mutually reachable two-commit cycle (1 -> 2 -> 1) must fail closed",
        );
        assert_corrupt_checkpoint(
            run_selector(&main, &[Some(CommitId(2)), Some(CommitId(1))], &cyclic),
            "mutually reachable two-commit cycle (reverse order) must fail closed",
        );
    }
}
