//! Reopen, custody, receipt, Bridge definition admission, and alternating-build
//! verification helpers for C4b.3 Signal+Bridge matrices.
use super::super::dimensions_matrix::{
    assert_cadence_checkpointed, assert_cadence_discarded, assert_cadence_retained, BuildDimension,
    DerivedDimension, SeededCoverage,
};
use super::super::ReopenPhase;
use super::assertions::{
    assert_receipt_bound_to_occurrence, assert_reopen_phase_snapshot, classify_signal_phase,
    SignalRef,
};
use super::child::SignalSession;
use super::wire::{SignalCmd, SignalReply, WireSnapshot};
use std::path::{Path, PathBuf};

pub(super) fn verify_signal_cell_reopen(
    location: &Path,
    dims: (BuildDimension, DerivedDimension),
    seed_info: (Option<SeededCoverage>, Option<u64>),
    check: (&SignalRef, &str),
) -> ReopenPhase {
    let ((build_dim, derived_dim), (seeded, mut active_discard)) = (dims, seed_info);
    let (sig_ref, label) = check;
    let builds = build_dim.sequence();
    let chans = tempfile::tempdir().unwrap();
    let mut seq = 0usize;
    let mut next_ch = |t: &str| {
        seq += 1;
        chans.path().join(format!("{seq}-{t}"))
    };

    let (mut session, opened1) = SignalSession::start(location, &next_ch("r1"), builds[0]);
    assert_eq!(
        (opened1.initialized, opened1.contacts),
        (0, (0, 0, 0, 0)),
        "{label}: reopen#1"
    );
    if derived_dim == DerivedDimension::DiscardedBeforeKill {
        assert_cadence_discarded(&opened1.cadence, &format!("{label}/r1"));
    } else {
        assert_cadence_retained(&opened1.cadence, seeded.unwrap(), &format!("{label}/r1"));
    }
    let phase = classify_signal_phase(&opened1, &sig_ref.before);
    assert_reopen_phase_snapshot(&opened1, phase, sig_ref, &format!("{label}/r1"));

    let opened = if derived_dim == DerivedDimension::DiscardedBetweenReopens {
        active_discard = session.send(SignalCmd::CheckpointAndDiscard).discard_gen;
        session.kill_live();
        let (s2, opened2) = SignalSession::start(location, &next_ch("r2"), builds[1]);
        session = s2;
        assert_eq!(
            (opened2.initialized, opened2.contacts),
            (0, (0, 0, 0, 0)),
            "{label}: reopen#2"
        );
        assert_cadence_discarded(&opened2.cadence, &format!("{label}/r2"));
        assert_eq!(
            classify_signal_phase(&opened2, &sig_ref.before),
            phase,
            "{label}: reopen#2 phase"
        );
        assert_eq!(
            opened2.snapshot, opened1.snapshot,
            "{label}: snapshot must survive D14 discard"
        );
        assert_eq!(
            opened2.custody, opened1.custody,
            "{label}: custody identity must survive D14 discard"
        );
        opened2
    } else {
        opened1
    };

    let (settled_snap, settled_contacts) =
        settle_signal_phase(&mut session, &opened, phase, sig_ref, label);
    assert!(
        settled_snap.same_durable_semantics(&sig_ref.after),
        "{label}: settled durable Signal/Bridge semantics parity"
    );
    // Explicitly observe & admit the recovered Bridge conditional definition
    // BEFORE publishing any new definitions, with zero provider contacts.
    let admitted = session.send(SignalCmd::VerifyRecoveredBridge);
    assert_eq!(
        admitted.definition_identity,
        Some(sig_ref.definition_identity),
        "{label}: recovered Bridge definition identity must match MemoryOnly before publish"
    );
    assert_eq!(
        admitted.contacts, settled_contacts,
        "{label}: admitting recovered Bridge definition must not call providers"
    );
    assert_eq!(
        admitted.snapshot.as_ref(),
        Some(&settled_snap),
        "{label}: admitting recovered Bridge definition must not mutate state"
    );

    let after_bridge = session
        .send(SignalCmd::VerifyBridgeHealthy {
            before_ord: settled_snap.world_commit_ordinal,
            before_sig_gen: sig_ref.after.signal_generation,
        })
        .snapshot
        .unwrap();
    assert_eq!(
        after_bridge.signal_generation,
        sig_ref.after.signal_generation + 2,
        "{label}: post-Bridge durable signal_generation parity"
    );
    assert_eq!(
        after_bridge.world_reference_generation,
        sig_ref.after.world_reference_generation + 2,
        "{label}: post-Bridge world_reference_generation parity"
    );

    finish_signal_builds(
        (location, &mut next_ch),
        (session, derived_dim, builds, active_discard),
        (&after_bridge, label),
    );
    phase
}

fn settle_signal_phase(
    session: &mut SignalSession,
    opened: &SignalReply,
    phase: ReopenPhase,
    sig_ref: &SignalRef,
    label: &str,
) -> (WireSnapshot, (usize, usize, usize, usize)) {
    let state = opened.snapshot.as_ref().unwrap();
    match phase {
        ReopenPhase::PreOp => {
            let r = session.send(SignalCmd::CommitMixed(2));
            assert_eq!(
                r.commit_tag.as_deref(),
                Some("committed"),
                "{label}: commit"
            );
            let after = r.snapshot.unwrap();
            let rec = r.receipt.expect("committed receipt");
            assert!(
                after.same_durable_semantics(&sig_ref.after),
                "{label}: preop semantics"
            );
            let expected_next = sig_ref.before.world_commit_ordinal + 1;
            assert!(
                after.world_commit_ordinal == expected_next
                    || after.world_commit_ordinal == expected_next + 1,
                "{label}: preop ordinal {} must be {expected_next} or {}",
                after.world_commit_ordinal,
                expected_next + 1
            );
            assert_receipt_bound_to_occurrence(&rec, &after, sig_ref, &format!("{label}/preop"));
            let dup = session.send(SignalCmd::CommitMixed(2));
            assert_eq!(dup.contacts, r.contacts, "{label}: dup retry 0 contacts");
            assert_eq!(dup.receipt, Some(rec), "{label}: dup retry exact receipt");
            assert_eq!(dup.snapshot.unwrap(), after, "{label}: dup retry snapshot");
            (after, dup.contacts)
        }
        ReopenPhase::Custody => {
            let custody = &opened.custody[0];
            let r = session.send(SignalCmd::CommitMixed(2));
            assert_eq!(
                r.commit_tag.as_deref(),
                Some("product_unpublished"),
                "{label}: custody retry"
            );
            assert_eq!(
                r.contacts,
                (0, 0, 0, 0),
                "{label}: custody retry 0 contacts"
            );
            let c = session.send(SignalCmd::ContinueCustody);
            assert_eq!(c.contacts, (0, 0, 0, 0), "{label}: continue 0 replay");
            let cont = c.snapshot.unwrap();
            assert!(
                cont.same_durable_semantics(&sig_ref.after),
                "{label}: cont semantics"
            );
            assert_eq!(
                cont.world_commit_ordinal,
                sig_ref.before.world_commit_ordinal + 1
            );
            let retry = session.send(SignalCmd::CommitMixed(2));
            assert_eq!(
                retry.commit_tag.as_deref(),
                Some("recovered_committed"),
                "{label}: post-continue retry"
            );
            assert_eq!(
                retry.contacts,
                (0, 0, 0, 0),
                "{label}: post-cont 0 contacts"
            );
            let rec = retry.receipt.expect("recovered receipt");
            assert_eq!(rec.attempt_ordinal, custody.attempt_ordinal);
            assert_eq!(Some(rec.commit_id), custody.native_commit_id);
            assert_eq!(Some(rec.version_id), custody.native_version_id);
            assert_receipt_bound_to_occurrence(&rec, &cont, sig_ref, &format!("{label}/custody"));
            assert_eq!(retry.snapshot.unwrap(), cont);
            (cont, retry.contacts)
        }
        ReopenPhase::Committed => {
            let retry = session.send(SignalCmd::CommitMixed(2));
            assert_eq!(
                retry.commit_tag.as_deref(),
                Some("recovered_committed"),
                "{label}: committed retry"
            );
            assert_eq!(retry.contacts, (0, 0, 0, 0), "{label}: zero replay");
            let rec = retry.receipt.expect("recovered receipt");
            assert_receipt_bound_to_occurrence(&rec, state, sig_ref, &format!("{label}/committed"));
            assert_eq!(retry.snapshot.unwrap(), *state);
            (state.clone(), retry.contacts)
        }
    }
}

fn finish_signal_builds(
    paths: (&Path, &mut dyn FnMut(&str) -> PathBuf),
    state: (SignalSession, DerivedDimension, [&str; 3], Option<u64>),
    check: (&WireSnapshot, &str),
) {
    let (location, next_ch) = paths;
    let (mut session, derived_dim, builds, active_discard) = state;
    let (final_snap, label) = check;
    let verify_checked = |loc: &Path, ch: &Path, build: &str, tag: &str| {
        let (mut s, o) = SignalSession::start(loc, ch, build);
        assert_eq!(
            (o.initialized, o.contacts),
            (0, (0, 0, 0, 0)),
            "{label}/{tag}: zero init/contacts"
        );
        assert_cadence_checkpointed(&o.cadence, &format!("{label}/{tag}"));
        assert_eq!(o.snapshot.unwrap(), *final_snap, "{label}/{tag}: snapshot");
        let c = s.send(SignalCmd::Close {
            expect_discarded: false,
        });
        assert!(c.closed, "{label}/{tag}: close");
        s.finish();
    };
    match derived_dim {
        DerivedDimension::DiscardedBeforeClose => {
            let discard_gen = session
                .send(SignalCmd::CheckpointAndDiscard)
                .discard_gen
                .unwrap();
            let closed = session.send(SignalCmd::Close {
                expect_discarded: true,
            });
            assert!(
                closed.closed && closed.close_gen.is_none(),
                "{label}: discarded close"
            );
            session.finish();
            let ch_disc = next_ch("r-disc");
            let (mut s2, o2) = SignalSession::start(location, &ch_disc, builds[1]);
            assert_eq!(
                (o2.initialized, o2.contacts),
                (0, (0, 0, 0, 0)),
                "{label}: post-discard reopen"
            );
            assert_cadence_discarded(&o2.cadence, &format!("{label}/post-discard"));
            assert_eq!(
                o2.snapshot.unwrap(),
                *final_snap,
                "{label}: post-discard snapshot"
            );
            let c2 = s2.send(SignalCmd::Close {
                expect_discarded: false,
            });
            assert_eq!(
                c2.close_gen,
                Some(discard_gen + 1),
                "{label}: resumed checkpoint gen"
            );
            s2.finish();
            let ch_fin = next_ch("r-fin");
            verify_checked(location, &ch_fin, builds[2], "final");
        }
        DerivedDimension::DiscardedBetweenReopens => {
            let c = session.send(SignalCmd::Close {
                expect_discarded: false,
            });
            assert!(c.closed, "{label}: normal close");
            assert_eq!(
                c.close_gen,
                active_discard.map(|d| d + 1),
                "{label}: discard_gen + 1"
            );
            session.finish();
            let ch_fin = next_ch("r-fin");
            verify_checked(location, &ch_fin, builds[2], "final");
        }
        DerivedDimension::Retained | DerivedDimension::DiscardedBeforeKill => {
            let c = session.send(SignalCmd::Close {
                expect_discarded: false,
            });
            assert!(c.closed, "{label}: normal close");
            if let Some(dg) = active_discard {
                assert_eq!(c.close_gen, Some(dg + 1), "{label}: discard_gen + 1");
            }
            session.finish();
            let ch_mid = next_ch("r-b1");
            verify_checked(location, &ch_mid, builds[1], "b1");
            let ch_fin = next_ch("r-b2");
            verify_checked(location, &ch_fin, builds[2], "b2");
        }
    }
}
