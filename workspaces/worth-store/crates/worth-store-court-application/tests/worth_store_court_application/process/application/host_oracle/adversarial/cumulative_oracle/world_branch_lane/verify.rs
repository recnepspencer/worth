//! Post-barrier World branch publish recovery, pure `MemoryOnly` payload
//! verification, and final checkpointed reopen for C4b.4 C2.8 lane.
use super::super::super::dimensions_matrix::assert_cadence_checkpointed;
use super::super::world_branch_child::{WbCmd, WorldBranchSession};
use super::super::{SplitMix64, StepCtx};
use super::setup::TopologySetup;
use std::path::Path;
use worth_execution::ExecutionRequest;

pub(super) struct BarrierTarget<'a> {
    pub(super) home_dir: &'a Path,
    pub(super) ch_post: &'a Path,
    pub(super) ch_fin: &'a Path,
    pub(super) branch: &'a str,
    pub(super) round: usize,
    pub(super) build: &'static str,
    pub(super) ctx: &'a StepCtx,
}

pub(super) fn verify_barrier_and_finalize(
    target: BarrierTarget<'_>,
    topo: &mut TopologySetup,
    rng: &mut SplitMix64,
    exec: &ExecutionRequest<'_, '_>,
) {
    let ctx_bar = target.ctx;
    let (mut s4, o4) = WorldBranchSession::start(target.home_dir, target.ch_post, target.build);
    let mut snap4 = o4.snap.unwrap();
    let prev = &topo.model.live[target.branch];
    assert_eq!(snap4.branches.len(), topo.model.live.len(), "{ctx_bar}");
    let actual = snap4
        .branches
        .iter()
        .find(|b| b.name == target.branch)
        .unwrap()
        .clone();
    for other in snap4.branches.iter().filter(|b| b.name != target.branch) {
        assert_eq!(other, &topo.model.live[&other.name], "{ctx_bar}: untouched");
    }
    let expected_entity = format!("entity-{}", target.round);
    let landed = if snap4.pending_custody > 0 {
        assert_eq!(snap4.pending_custody, 1, "{ctx_bar}: single pending effect");
        assert_eq!(&actual, prev, "{ctx_bar}: custody keeps predecessor");
        topo.mem_ref.assert_payload_parity(&snap4, exec, ctx_bar);
        let cont_reply = s4.send(WbCmd::ContinuePendingCustody);
        let continued = cont_reply.custody_continued.unwrap();
        snap4 = cont_reply.snap.unwrap();
        assert_eq!(snap4.pending_custody, 0, "{ctx_bar}: custody cleared");
        continued
    } else {
        &actual != prev
    };
    if landed {
        topo.mem_ref.publish_on(target.branch, target.round, exec);
        let after = snap4
            .branches
            .iter()
            .find(|b| b.name == target.branch)
            .unwrap();
        assert_eq!(after.reference_gen, prev.reference_gen + 1, "{ctx_bar}");
        assert!(after.commit_ordinal > prev.commit_ordinal, "{ctx_bar}");
        assert_eq!(
            snap4.installed_commits,
            topo.model.installed_commits + 1,
            "{ctx_bar}: landed installed_commits + 1"
        );
        assert!(
            after
                .entity_payloads
                .iter()
                .any(|p| p.contains(&expected_entity)),
            "{ctx_bar}: landed successor must contain exact owner payload {expected_entity}"
        );
        topo.mem_ref.assert_payload_parity(&snap4, exec, ctx_bar);
    } else {
        let after = snap4
            .branches
            .iter()
            .find(|b| b.name == target.branch)
            .unwrap();
        assert_eq!(after, prev, "{ctx_bar}: pre-op/released keeps predecessor");
        assert_eq!(
            snap4.installed_commits, topo.model.installed_commits,
            "{ctx_bar}: unlanded installed_commits unchanged"
        );
        assert!(
            !after
                .entity_payloads
                .iter()
                .any(|p| p.contains(&expected_entity)),
            "{ctx_bar}: unlanded branch must not expose {expected_entity}"
        );
        topo.mem_ref.assert_payload_parity(&snap4, exec, ctx_bar);
    }

    let disposed_list: Vec<u64> = topo.model.disposed.iter().copied().collect();
    s4.send(WbCmd::AssertDisposed {
        ordinals: disposed_list.clone(),
    });
    let closed = s4.send(WbCmd::Close {
        expect_discarded: false,
    });
    assert!(closed.closed && closed.close_gen.is_some(), "{ctx_bar}");
    s4.finish();

    let fin_build = rng.next_other_build(target.build);
    let (mut fin, ofin) = WorldBranchSession::start(target.home_dir, target.ch_fin, fin_build);
    let fin_snap = ofin.snap.unwrap();
    assert_cadence_checkpointed(&fin_snap.cadence, &format!("{ctx_bar}/final"));
    assert_eq!(fin_snap.branches, snap4.branches, "{ctx_bar}: final");
    topo.mem_ref.assert_payload_parity(&fin_snap, exec, ctx_bar);
    fin.send(WbCmd::AssertDisposed {
        ordinals: disposed_list,
    });
    let mut protected = topo.protected_ancestors.clone();
    protected.extend(fin_snap.branches.iter().map(|b| b.commit_ordinal));
    fin.send(WbCmd::AssertProtectedInAncestry {
        ordinals: protected,
    });
    assert!(
        fin.send(WbCmd::Close {
            expect_discarded: false,
        })
        .closed
    );
    fin.finish();
}
