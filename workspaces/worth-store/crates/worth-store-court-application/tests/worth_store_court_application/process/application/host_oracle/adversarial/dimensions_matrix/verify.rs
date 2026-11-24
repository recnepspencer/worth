//! Session cadence, discard, settlement, and alternating-build verification
//! helpers for C4b.3 Host matrices.
use super::super::super::{
    protocol::{Disposition, Request},
    session::Session,
};
use super::super::oracle_verifier::{
    ReferenceSnapshots, FOLLOW_KEY, FOLLOW_VAL, TARGET_KEY, TARGET_VAL,
};
use super::super::BUILD_PRIMARY;
use super::{
    assert_cadence_checkpointed, assert_cadence_discarded, DerivedDimension, SeededCoverage,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use worth_store_testkit_application_oracle::facade::host_observation::{
    adversarial_verdict::{verify_continued_parity, ContinuedParityCheck, RecoveredPrefix},
    comparison::{self, HostOracleComparison},
    projection::{CustodyProjection, StateProjection},
    receipts::{self, MutationFate},
};

pub(in super::super) fn inspect_cadence(session: &mut Session) -> BTreeMap<String, (u64, u64)> {
    let resp = session.send(Request::InspectCadence);
    let Some(Disposition::Cadence { journals }) = resp.disposition else {
        panic!("expected Cadence disposition, got {:?}", resp.disposition);
    };
    journals.into_iter().map(|(k, c, h)| (k, (c, h))).collect()
}

pub(super) fn capture_seeded_coverage(
    home_dir: &Path,
    channel: &Path,
    label: &str,
) -> SeededCoverage {
    let (mut s, opened) = Session::start_with_build(home_dir, channel, 0, Some(BUILD_PRIMARY));
    assert_eq!(
        (opened.initialized, opened.contacts),
        (0, 0),
        "{label}: seeded cadence open"
    );
    let cadence = inspect_cadence(&mut s);
    assert_cadence_checkpointed(&cadence, &format!("{label}/seeded"));
    let seeded = SeededCoverage {
        native_covered: cadence["relational.canonical-commits"].0,
        world_covered: cadence["world.occurrences"].0,
    };
    close_normal(&mut s, None, &format!("{label}/seeded-close"));
    seeded
}

pub(in super::super) fn checkpoint_and_discard(session: &mut Session) -> u64 {
    let resp = session.send(Request::CheckpointAndDiscard);
    let Some(Disposition::Discarded {
        checkpoint_generation,
        discard_generation,
    }) = resp.disposition
    else {
        panic!("expected Discarded disposition, got {:?}", resp.disposition);
    };
    assert_eq!(discard_generation, checkpoint_generation + 1);
    discard_generation
}

pub(in super::super) fn close_expecting_discarded(session: &mut Session, label: &str) {
    let resp = session.send(Request::CloseExpectingDiscarded);
    assert!(resp.closed, "{label}: CloseExpectingDiscarded must close");
    assert_eq!(
        resp.disposition,
        Some(Disposition::ClosedWithCheckpoint { generation: None }),
        "{label}: discarded close must not publish checkpoint"
    );
    session.finish();
}

pub(in super::super) fn close_normal(
    session: &mut Session,
    expected_after_discard: Option<u64>,
    label: &str,
) {
    let resp = session.send(Request::Close);
    assert!(resp.closed, "{label}: normal close must close");
    let Some(Disposition::ClosedWithCheckpoint {
        generation: Some(cp_gen),
    }) = resp.disposition
    else {
        panic!(
            "{label}: normal close must publish checkpoint, got {:?}",
            resp.disposition
        );
    };
    if let Some(discard_gen) = expected_after_discard {
        assert_eq!(
            cp_gen,
            discard_gen + 1,
            "{label}: fresh checkpoint after discard must advance discard_generation + 1"
        );
    }
    session.finish();
}

pub(super) fn assert_same_custody_rows(
    first: &[CustodyProjection],
    second: &[CustodyProjection],
    label: &str,
) {
    assert_eq!(
        first.len(),
        second.len(),
        "{label}: custody row count across D14 discard"
    );
    for (idx, (a, b)) in first.iter().zip(second).enumerate() {
        assert_eq!(
            (
                a.owner,
                a.identity,
                a.effects,
                a.settlement,
                &a.cause,
                &a.expected,
                &a.actions,
                &a.original_effect
            ),
            (
                b.owner,
                b.identity,
                b.effects,
                b.settlement,
                &b.cause,
                &b.expected,
                &b.actions,
                &b.original_effect
            ),
            "{label}: custody row #{idx} mutated across D14 discard"
        );
    }
}

pub(super) fn settle_drift_and_follow(
    session: &mut Session,
    map: &mut HostOracleComparison,
    verdict: &RecoveredPrefix,
    reopened: &StateProjection,
    reference: &ReferenceSnapshots,
    label: &str,
) {
    let expected_contacts = match verdict {
        RecoveredPrefix::PreOperation => {
            let first = session.send(Request::Mutate {
                key: TARGET_KEY,
                value: TARGET_VAL,
            });
            let first_obs = first.mutation.unwrap();
            let after = first.state.unwrap();
            assert_eq!(
                (first_obs.fate, first.contacts),
                (MutationFate::Committed, 1),
                "{label}: preop mutate"
            );
            receipts::compare(
                map.identities_mut(),
                &first_obs,
                &reference.target_receipt,
                &after,
                &reference.after,
            )
            .unwrap_or_else(|e| panic!("{label}: first receipt: {e:?}"));
            comparison::compare(map, &after, &reference.after)
                .unwrap_or_else(|e| panic!("{label}: after state: {e:?}"));
            let retry = session.send(Request::Mutate {
                key: TARGET_KEY,
                value: TARGET_VAL,
            });
            assert_eq!(
                retry.contacts, 1,
                "{label}: duplicate retry 0 extra contacts"
            );
            assert_eq!(
                retry.mutation.unwrap(),
                first_obs,
                "{label}: duplicate retry receipt"
            );
            1
        }
        RecoveredPrefix::UnpublishedCustody {
            attempt_owner,
            attempt,
            original_native,
            ..
        } => {
            let retry = session.send(Request::Mutate {
                key: TARGET_KEY,
                value: TARGET_VAL,
            });
            assert_eq!(
                (retry.mutation.unwrap().fate, retry.contacts),
                (MutationFate::ProductUnpublished, 0),
                "{label}: custody retry"
            );
            let cont = session.send(Request::ContinueOriginal {
                key: TARGET_KEY,
                value: TARGET_VAL,
                pause_after_disposition: false,
            });
            assert_eq!(cont.contacts, 0, "{label}: continue 0 contacts");
            let Some(Disposition::Continued(rec_obs)) = cont.disposition else {
                panic!("{label}: expected Continued disposition");
            };
            let continued_state = cont.state.unwrap();
            verify_continued_parity(&ContinuedParityCheck {
                continued: &continued_state,
                after_ref: &reference.after,
                before_head: &reopened.head,
                attempt_owner: *attempt_owner,
                attempt: *attempt,
                original_native,
                receipt: rec_obs.receipt.as_ref().unwrap(),
                reference_receipt: reference.target_receipt.receipt.as_ref().unwrap(),
            })
            .unwrap_or_else(|e| panic!("{label}: continued parity: {e:?}"));
            comparison::compare(map, &continued_state, &reference.after)
                .unwrap_or_else(|e| panic!("{label}: continued correspondence: {e:?}"));
            // Retry the admitted key after continuation: exact same receipt, 0 handler contacts.
            let post_retry = session.send(Request::Mutate {
                key: TARGET_KEY,
                value: TARGET_VAL,
            });
            assert_eq!(
                post_retry.contacts, 0,
                "{label}: post-continuation retry 0 contacts"
            );
            assert_eq!(
                post_retry.mutation.unwrap(),
                rec_obs,
                "{label}: post-continuation retry must return exact continued receipt"
            );
            assert_eq!(
                post_retry.state.unwrap().head,
                continued_state.head,
                "{label}: post-continuation retry must not advance head"
            );
            0
        }
        RecoveredPrefix::CommittedInFlight => {
            let retry = session.send(Request::Mutate {
                key: TARGET_KEY,
                value: TARGET_VAL,
            });
            assert_eq!(retry.contacts, 0, "{label}: committed retry 0 contacts");
            let retry_obs = retry.mutation.unwrap();
            assert_eq!(
                retry_obs.fate,
                MutationFate::Committed,
                "{label}: committed retry fate"
            );
            receipts::compare(
                map.identities_mut(),
                &retry_obs,
                &reference.target_receipt,
                reopened,
                &reference.after,
            )
            .unwrap_or_else(|e| panic!("{label}: recovered receipt: {e:?}"));
            0
        }
    };
    let drift = session.send(Request::Mutate {
        key: TARGET_KEY,
        value: TARGET_VAL + 77,
    });
    let drift_obs = drift.mutation.unwrap();
    let after_target = drift.state.unwrap();
    assert_eq!(
        (drift_obs.fate, drift.contacts),
        (MutationFate::IntentDrift, expected_contacts),
        "{label}: drift"
    );
    receipts::compare(
        map.identities_mut(),
        &drift_obs,
        &reference.drift_receipt,
        &after_target,
        &reference.after,
    )
    .unwrap_or_else(|e| panic!("{label}: drift receipt: {e:?}"));

    let follow = session.send(Request::Mutate {
        key: FOLLOW_KEY,
        value: FOLLOW_VAL,
    });
    let follow_obs = follow.mutation.unwrap();
    let final_state = follow.state.unwrap();
    assert_eq!(
        (follow_obs.fate, follow.contacts),
        (MutationFate::Committed, expected_contacts + 1),
        "{label}: follow"
    );
    receipts::compare(
        map.identities_mut(),
        &follow_obs,
        &reference.follow_receipt,
        &final_state,
        &reference.after_follow,
    )
    .unwrap_or_else(|e| panic!("{label}: follow receipt: {e:?}"));
    comparison::compare(map, &final_state, &reference.after_follow)
        .unwrap_or_else(|e| panic!("{label}: final parity: {e:?}"));
}

pub(super) fn verify_final_reopen(
    home_dir: &Path,
    channel: &Path,
    build: &str,
    map: &mut HostOracleComparison,
    reference: &ReferenceSnapshots,
    label: &str,
) {
    let (mut session, opened) = Session::start_with_build(home_dir, channel, 0, Some(build));
    assert_eq!(
        (opened.initialized, opened.contacts),
        (0, 0),
        "{label}: final reopen"
    );
    assert_cadence_checkpointed(&inspect_cadence(&mut session), &format!("{label}/final"));
    comparison::compare(map, &opened.state.unwrap(), &reference.after_follow)
        .unwrap_or_else(|e| panic!("{label}: final reopen parity: {e:?}"));
    close_normal(&mut session, None, &format!("{label}/final-close"));
}

/// Complete every cell so ALL THREE builds (`builds[0]`, `builds[1]`, `builds[2]`)
/// are genuinely reopened in fresh OS processes (never skipping tertiary build).
pub(super) fn finish_cell_builds(
    paths: (&Path, &mut dyn FnMut(&str) -> PathBuf),
    state: (Session, DerivedDimension, [&str; 3], Option<u64>),
    check: (&mut HostOracleComparison, &ReferenceSnapshots, &str),
) {
    let (home_dir, next_chan) = paths;
    let (mut session, derived_dim, builds, active_discard_gen) = state;
    let (map, reference, label) = check;
    match derived_dim {
        DerivedDimension::DiscardedBeforeClose => {
            let discard_gen = checkpoint_and_discard(&mut session);
            close_expecting_discarded(&mut session, label);
            let ch_disc = next_chan("post-discard");
            let (mut s2, o2) = Session::start_with_build(home_dir, &ch_disc, 0, Some(builds[1]));
            assert_eq!(
                (o2.initialized, o2.contacts),
                (0, 0),
                "{label}: post-discard reopen"
            );
            assert_cadence_discarded(&inspect_cadence(&mut s2), &format!("{label}/post-discard"));
            comparison::compare(map, &o2.state.unwrap(), &reference.after_follow)
                .unwrap_or_else(|e| panic!("{label}: post-discard parity: {e:?}"));
            close_normal(&mut s2, Some(discard_gen), &format!("{label}/resume-cp"));
            let ch_final = next_chan("final");
            verify_final_reopen(home_dir, &ch_final, builds[2], map, reference, label);
        }
        DerivedDimension::DiscardedBetweenReopens => {
            close_normal(&mut session, active_discard_gen, &format!("{label}/close"));
            let ch_final = next_chan("final");
            verify_final_reopen(home_dir, &ch_final, builds[2], map, reference, label);
        }
        DerivedDimension::Retained | DerivedDimension::DiscardedBeforeKill => {
            close_normal(&mut session, active_discard_gen, &format!("{label}/close"));
            let ch_mid = next_chan("mid-build");
            verify_final_reopen(
                home_dir,
                &ch_mid,
                builds[1],
                map,
                reference,
                &format!("{label}/b1"),
            );
            let ch_final = next_chan("final");
            verify_final_reopen(
                home_dir,
                &ch_final,
                builds[2],
                map,
                reference,
                &format!("{label}/b2"),
            );
        }
    }
}
