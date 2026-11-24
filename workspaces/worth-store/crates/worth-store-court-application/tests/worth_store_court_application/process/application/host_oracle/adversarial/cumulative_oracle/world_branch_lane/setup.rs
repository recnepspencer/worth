//! Expected World branch model, pure topology-derived exact reclaim sets,
//! & Phase 1/2 multi-topology setup for C4b.4 C2.8 lane.
use super::super::world_branch_child::{BranchView, WbCmd, WorldBranchSession, WorldSnap};
use super::super::{SplitMix64, StepCtx};
pub(super) use super::memory_ref::MemoryOnlyWorldRef;
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::ExecutionRequest;

pub(super) const ROOT_NAME: &str = "Original World Root";

pub(super) struct ExpectedWorldModel {
    pub(super) live: BTreeMap<String, BranchView>,
    pub(super) retired_readable: BTreeSet<u64>,
    pub(super) disposed: BTreeSet<u64>,
    pub(super) installed_commits: usize,
}

impl ExpectedWorldModel {
    pub(super) fn from_initial(snap: &WorldSnap) -> Self {
        let live = snap
            .branches
            .iter()
            .map(|b| (b.name.clone(), b.clone()))
            .collect();
        Self {
            live,
            retired_readable: BTreeSet::new(),
            disposed: BTreeSet::new(),
            installed_commits: snap.installed_commits,
        }
    }

    pub(super) fn record_branch(&mut self, snap: &WorldSnap, name: &str) -> u64 {
        let view = snap
            .branches
            .iter()
            .find(|b| b.name == name)
            .unwrap_or_else(|| panic!("expected branch {name} in snapshot"))
            .clone();
        let ord = view.commit_ordinal;
        self.live.insert(name.to_owned(), view);
        self.installed_commits = snap.installed_commits;
        ord
    }

    pub(super) fn assert_matches(&self, snap: &WorldSnap, ctx: &StepCtx) {
        assert_eq!(snap.pending_custody, 0, "{ctx}: expected 0 pending custody");
        assert_eq!(snap.branches.len(), self.live.len(), "{ctx}: branch count");
        for actual in &snap.branches {
            let expected = self
                .live
                .get(&actual.name)
                .unwrap_or_else(|| panic!("{ctx}: unexpected live branch {}", actual.name));
            assert_eq!(actual, expected, "{ctx}: branch {} divergence", actual.name);
        }
        assert_eq!(
            snap.installed_commits, self.installed_commits,
            "{ctx}: installed commit count mismatch"
        );
    }
}

pub(super) struct TopologySetup {
    pub(super) mem_ref: MemoryOnlyWorldRef,
    pub(super) model: ExpectedWorldModel,
    pub(super) parent_to_retire: String,
    pub(super) sib_exclusive: BTreeSet<u64>,
    pub(super) sib_retired_head: u64,
    /// Exact changed-history reclaim set derived purely from the fixture commit DAG
    /// (zero Directory self-comparison): every non-head, non-branch-point interior
    /// commit with a sole installed child after `sib_exclusive` is retired.
    pub(super) expected_changed_set: BTreeSet<u64>,
    pub(super) protected_ancestors: Vec<u64>,
    pub(super) protected_live_heads: Vec<u64>,
    pub(super) round: usize,
    pub(super) branch_seq: usize,
}

fn fork_both(
    session: &mut WorldBranchSession,
    mem_ref: &mut MemoryOnlyWorldRef,
    model: &mut ExpectedWorldModel,
    parent: &str,
    name: &str,
    prefix: &str,
    nfork: bool,
    sfork: bool,
    exec: &ExecutionRequest<'_, '_>,
) -> (WorldSnap, u64) {
    let snap = session
        .send(WbCmd::CreateBranch {
            parent: parent.to_owned(),
            name: name.to_owned(),
            fork_prefix: prefix.to_owned(),
            native_fork: nfork,
            signal_fork: sfork,
        })
        .snap
        .unwrap();
    mem_ref.create_branch(parent, name, &format!("{prefix}-native"), nfork, exec);
    let ord = model.record_branch(&snap, name);
    (snap, ord)
}

pub(super) fn run_topology_setup(
    session: &mut WorldBranchSession,
    rng: &mut SplitMix64,
    seed: u64,
    build: &'static str,
) -> TopologySetup {
    let req = crate::workload::serial_request();
    let exec = ExecutionRequest::serial(&req);
    let mut mem_ref = MemoryOnlyWorldRef::new();
    let (mut round, mut branch_seq) = (0usize, 0usize);

    let boot = session.send(WbCmd::BootstrapRoot).snap.unwrap();
    let boot_ord = boot.branches[0].commit_ordinal;
    let pub0 = session
        .send(WbCmd::Publish {
            branch: ROOT_NAME.to_owned(),
            round,
        })
        .snap
        .unwrap();
    mem_ref.publish_on(ROOT_NAME, round, &exec);
    round += 1;
    let mut model = ExpectedWorldModel::from_initial(&pub0);
    let shared_fork_point = model.live[ROOT_NAME].commit_ordinal;
    assert_eq!((boot.branches.len(), model.live.len()), (1, 1));

    // Phase 1: Verify all 3 shared-component fork combinations in PRNG-shuffled order.
    let mut shared_combos = [
        ("n-only", true, false),
        ("s-only", false, true),
        ("reuse", false, false),
    ];
    for i in (1..shared_combos.len()).rev() {
        shared_combos.swap(i, rng.range(i + 1));
    }
    for (idx, (tag, nfork, sfork)) in shared_combos.into_iter().enumerate() {
        branch_seq += 1;
        let name = format!("s{seed}-b{branch_seq}-{tag}");
        let prefix = format!("s{seed}-p{branch_seq}-{tag}");
        let ctx = StepCtx {
            seed,
            step: idx + 1,
            family: "c2.8-world-branches",
            op: format!("shared-combo({name})"),
            cut: "none".into(),
            build,
            derived: "retained",
        };
        let (snap, _) = fork_both(
            session,
            &mut mem_ref,
            &mut model,
            ROOT_NAME,
            &name,
            &prefix,
            nfork,
            sfork,
            &exec,
        );
        let root_view = &model.live[ROOT_NAME];
        let created_view = &model.live[&name];
        assert_eq!(
            created_view.native_branch != root_view.native_branch,
            nfork,
            "{ctx}"
        );
        assert_eq!(
            created_view.signal_branch != root_view.signal_branch,
            sfork,
            "{ctx}"
        );
        model.assert_matches(&snap, &ctx);
        mem_ref.assert_payload_parity(&snap, &exec, &ctx);
        let ret = session.send(WbCmd::RetireBranch { name: name.clone() });
        mem_ref.retire_branch(&name);
        model.live.remove(&name);
        model.assert_matches(ret.snap.as_ref().unwrap(), &ctx);
    }

    // Phase 2: Build exact known commit DAG on ROOT -> parent_to_retire -> child_name.
    branch_seq += 1;
    let parent_to_retire = format!("s{seed}-sib-{branch_seq}");
    let sib_prefix = format!("s{seed}-sibp-{branch_seq}");
    let (_, sib_fork_ord) = fork_both(
        session,
        &mut mem_ref,
        &mut model,
        ROOT_NAME,
        &parent_to_retire,
        &sib_prefix,
        true,
        true,
        &exec,
    );
    let p_sib0 = session
        .send(WbCmd::Publish {
            branch: parent_to_retire.clone(),
            round,
        })
        .snap
        .unwrap();
    mem_ref.publish_on(&parent_to_retire, round, &exec);
    round += 1;
    let sib_pub0_ord = model.record_branch(&p_sib0, &parent_to_retire);

    branch_seq += 1;
    let child_name = format!("s{seed}-child-{branch_seq}");
    let child_prefix = format!("s{seed}-childp-{branch_seq}");
    let (mut last_snap, child_fork_ord) = fork_both(
        session,
        &mut mem_ref,
        &mut model,
        &parent_to_retire,
        &child_name,
        &child_prefix,
        true,
        true,
        &exec,
    );
    let (mut sib_exclusive, mut root_post_fork) = (Vec::new(), Vec::new());
    let mut ops = [
        ROOT_NAME.to_owned(),
        ROOT_NAME.to_owned(),
        parent_to_retire.clone(),
        parent_to_retire.clone(),
        child_name.clone(),
    ];
    for i in (1..ops.len()).rev() {
        ops.swap(i, rng.range(i + 1));
    }
    for target in ops {
        let p = session
            .send(WbCmd::Publish {
                branch: target.clone(),
                round,
            })
            .snap
            .unwrap();
        mem_ref.publish_on(&target, round, &exec);
        round += 1;
        let ord = model.record_branch(&p, &target);
        last_snap = p;
        if target == parent_to_retire {
            sib_exclusive.push(ord);
        } else if target == ROOT_NAME {
            root_post_fork.push(ord);
        }
    }
    let sib_retired_head = *sib_exclusive.last().unwrap();
    let protected_live_heads = vec![
        model.live[ROOT_NAME].commit_ordinal,
        model.live[&child_name].commit_ordinal,
    ];
    let protected_ancestors = vec![
        shared_fork_point,
        model.live[ROOT_NAME].commit_ordinal,
        model.live[&child_name].commit_ordinal,
    ];
    // Pure topology-derived exact changed-history reclaim set after `sib_exclusive` is retired.
    let mut expected_changed_set = BTreeSet::new();
    for ord in [
        boot_ord,
        sib_fork_ord,
        sib_pub0_ord,
        child_fork_ord,
        root_post_fork[0],
    ] {
        if ord != shared_fork_point && !protected_live_heads.contains(&ord) {
            expected_changed_set.insert(ord);
        }
    }
    let setup_ctx = StepCtx {
        seed,
        step: 9,
        family: "c2.8-world-branches",
        op: "topology-setup-complete".into(),
        cut: "none".into(),
        build,
        derived: "retained",
    };
    mem_ref.assert_payload_parity(&last_snap, &exec, &setup_ctx);
    TopologySetup {
        mem_ref,
        model,
        parent_to_retire,
        sib_exclusive: sib_exclusive.into_iter().collect(),
        sib_retired_head,
        expected_changed_set,
        protected_ancestors,
        protected_live_heads,
        round,
        branch_seq,
    }
}
