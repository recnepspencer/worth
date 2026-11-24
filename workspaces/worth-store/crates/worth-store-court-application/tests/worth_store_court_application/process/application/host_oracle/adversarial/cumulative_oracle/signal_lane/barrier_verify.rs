//! Post-barrier Signal verification and final checkpointed reopen helper.
use super::super::super::dimensions_matrix::assert_cadence_checkpointed;
use super::super::super::signal_bridge_io_kills::{commit_mixed, SignalHome};
use super::super::{SplitMix64, StepCtx};
use super::child::CumSignalSession;
use super::ops::{
    assert_multi_parity, assert_receipt_bound, observe_multi_branch, sorted_branches, CumSignalCmd,
};
use std::path::Path;

pub(super) struct SignalBarrierCheck<'a> {
    pub(super) home_dir: &'a Path,
    pub(super) ch_post: &'a Path,
    pub(super) ch_fin: &'a Path,
    pub(super) branch_idx: usize,
    pub(super) target_ord: u8,
    pub(super) expected_def_id: [u8; 32],
    pub(super) build: &'static str,
    pub(super) ctx: &'a StepCtx,
}

pub(super) fn verify_signal_barrier_and_finalize(
    check: SignalBarrierCheck<'_>,
    ref_home: &SignalHome,
    rng: &mut SplitMix64,
) {
    let (tb, target_ord, ctx_bar) = (check.branch_idx, check.target_ord, check.ctx);
    let (mut s3, o3) = CumSignalSession::start(check.home_dir, check.ch_post, check.build);
    assert_eq!(
        (o3.initialized, o3.contacts),
        (0, (0, 0, 0, 0)),
        "{ctx_bar}"
    );
    let ref_before = observe_multi_branch(ref_home);
    let ref_br = sorted_branches(ref_home)[tb];
    let snap3 = o3.snap.as_ref().unwrap();
    if !o3.custody.is_empty() {
        assert_eq!(o3.custody.len(), 1, "{ctx_bar}: single custody row");
        assert_multi_parity(snap3, &ref_before, ctx_bar);
        let unpub = s3.send(CumSignalCmd::CommitMixed {
            branch_idx: tb,
            ordinal: target_ord,
        });
        assert_eq!(
            unpub.commit_tag.as_deref(),
            Some("product_unpublished"),
            "{ctx_bar}"
        );
        assert_eq!(
            unpub.contacts,
            (0, 0, 0, 0),
            "{ctx_bar}: custody 0 contacts"
        );
        let cont = s3.send(CumSignalCmd::ContinueCustody);
        assert_eq!(
            cont.contacts,
            (0, 0, 0, 0),
            "{ctx_bar}: continue 0 contacts"
        );
        assert!(commit_mixed(ref_home, ref_br, target_ord)
            .require_committed()
            .is_ok());
        let cont_snap = cont.snap.as_ref().unwrap();
        assert_multi_parity(cont_snap, &observe_multi_branch(ref_home), ctx_bar);
        let r = s3.send(CumSignalCmd::CommitMixed {
            branch_idx: tb,
            ordinal: target_ord,
        });
        assert_eq!(
            r.commit_tag.as_deref(),
            Some("recovered_committed"),
            "{ctx_bar}"
        );
        assert_eq!(r.contacts, (0, 0, 0, 0), "{ctx_bar}: post-cont 0 contacts");
        assert_receipt_bound(
            r.receipt.as_ref().unwrap(),
            target_ord,
            &cont_snap.branches[tb],
            ctx_bar,
        );
    } else if snap3.branches[tb].intent == ref_before.branches[tb].intent {
        assert_multi_parity(snap3, &ref_before, ctx_bar);
        assert!(commit_mixed(ref_home, ref_br, target_ord)
            .require_committed()
            .is_ok());
        let r = s3.send(CumSignalCmd::CommitMixed {
            branch_idx: tb,
            ordinal: target_ord,
        });
        assert_eq!(
            r.commit_tag.as_deref(),
            Some("committed"),
            "{ctx_bar}: preop commit"
        );
        let rsnap = r.snap.as_ref().unwrap();
        assert_receipt_bound(
            r.receipt.as_ref().unwrap(),
            target_ord,
            &rsnap.branches[tb],
            ctx_bar,
        );
        assert_multi_parity(rsnap, &observe_multi_branch(ref_home), ctx_bar);
    } else {
        assert!(commit_mixed(ref_home, ref_br, target_ord)
            .require_committed()
            .is_ok());
        assert_multi_parity(snap3, &observe_multi_branch(ref_home), ctx_bar);
        let r = s3.send(CumSignalCmd::CommitMixed {
            branch_idx: tb,
            ordinal: target_ord,
        });
        assert_eq!(
            r.commit_tag.as_deref(),
            Some("recovered_committed"),
            "{ctx_bar}"
        );
        assert_eq!(r.contacts, (0, 0, 0, 0), "{ctx_bar}: committed 0 contacts");
        assert_receipt_bound(
            r.receipt.as_ref().unwrap(),
            target_ord,
            &snap3.branches[tb],
            ctx_bar,
        );
    }
    let pre_close = s3.send(CumSignalCmd::VerifyRecoveredBridge { branch_idx: tb });
    assert_eq!(
        pre_close.definition_identity,
        Some(check.expected_def_id),
        "{ctx_bar}"
    );
    let closed = s3.send(CumSignalCmd::Close {
        expect_discarded: false,
    });
    assert!(closed.closed && closed.close_gen.is_some(), "{ctx_bar}");
    s3.finish();

    let fin_build = rng.next_other_build(check.build);
    let (mut fin, o_fin) = CumSignalSession::start(check.home_dir, check.ch_fin, fin_build);
    assert_eq!(
        (o_fin.initialized, o_fin.contacts),
        (0, (0, 0, 0, 0)),
        "{ctx_bar}: final"
    );
    assert_cadence_checkpointed(&o_fin.cadence, &format!("{ctx_bar}/final"));
    assert_multi_parity(
        o_fin.snap.as_ref().unwrap(),
        &observe_multi_branch(ref_home),
        ctx_bar,
    );
    assert!(
        fin.send(CumSignalCmd::Close {
            expect_discarded: false,
        })
        .closed
    );
    fin.finish();
}
