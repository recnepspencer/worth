//! C4b.3: Orthogonal build-identity change (Same / Changed / Alternating) and
//! D14 derived-state discard (Retained / BeforeKill / BetweenReopens /
//! BeforeClose) matrix over real OS mid-operation kills, EVERY Store mutation
//! seam, and EVERY physical/opening recovery stage in fresh OS processes.
mod cadence;
pub(super) mod recovery;
pub(super) mod verify;

pub(super) use self::cadence::{
    assert_cadence_checkpointed, assert_cadence_discarded, assert_cadence_retained,
    EXPECTED_CADENCE_KEYS,
};
use self::verify::{
    assert_same_custody_rows, capture_seeded_coverage, checkpoint_and_discard,
    close_expecting_discarded, finish_cell_builds, inspect_cadence, settle_drift_and_follow,
};
use super::super::{opening, session::Session};
use super::oracle_verifier::{
    build_reference, prepare_prefix, ReferenceSnapshots, TARGET_KEY, TARGET_VAL,
};
use super::recovery_fixtures::{discover_mutate_ordinals, MutateSeamOrdinals};
pub(super) use super::run_parallel_items;
use super::seam_kill_sweep::{kill_at_seam_with_ctx, measure_appends, ALL_SEAMS};
use super::{RestorableHome, BUILD_PRIMARY, BUILD_SECONDARY, BUILD_TERTIARY};
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_store_testkit_application_oracle::facade::host_observation::{
    adversarial_verdict::{classify_and_verify_reopen, RecoveredPrefix},
    comparison::HostOracleComparison,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SeededCoverage {
    pub(super) native_covered: u64,
    pub(super) world_covered: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BuildDimension {
    Same,
    Changed,
    Alternating,
}

impl BuildDimension {
    pub(super) const ALL: [Self; 3] = [Self::Same, Self::Changed, Self::Alternating];

    pub(super) fn sequence(self) -> [&'static str; 3] {
        match self {
            Self::Same => [BUILD_PRIMARY, BUILD_PRIMARY, BUILD_PRIMARY],
            Self::Changed => [BUILD_SECONDARY, BUILD_SECONDARY, BUILD_SECONDARY],
            Self::Alternating => [BUILD_SECONDARY, BUILD_TERTIARY, BUILD_PRIMARY],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DerivedDimension {
    Retained,
    DiscardedBeforeKill,
    DiscardedBetweenReopens,
    DiscardedBeforeClose,
}

impl DerivedDimension {
    pub(super) const ALL: [Self; 4] = [
        Self::Retained,
        Self::DiscardedBeforeKill,
        Self::DiscardedBetweenReopens,
        Self::DiscardedBeforeClose,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Posture {
    PreOp,
    Custody,
    Committed,
}

impl Posture {
    const ALL: [Self; 3] = [Self::PreOp, Self::Custody, Self::Committed];

    fn ordinal(self, ord: MutateSeamOrdinals) -> usize {
        match self {
            Self::PreOp => ord.pre_op_after_wal,
            Self::Custody => ord.custody_after_wal,
            Self::Committed => ord.committed_after_wal,
        }
    }

    fn assert_matches(self, verdict: &RecoveredPrefix, label: &str) {
        match self {
            Self::PreOp => assert_eq!(*verdict, RecoveredPrefix::PreOperation, "{label}"),
            Self::Custody => assert!(
                matches!(verdict, RecoveredPrefix::UnpublishedCustody { .. }),
                "{label}: {verdict:?}"
            ),
            Self::Committed => assert_eq!(*verdict, RecoveredPrefix::CommittedInFlight, "{label}"),
        }
    }
}

#[test]
fn host_mid_operation_kills_across_build_and_d14_discard_matrix() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let ordinals = discover_mutate_ordinals(&execution);

    let mut cells = Vec::with_capacity(36);
    for posture in Posture::ALL {
        for build_dim in BuildDimension::ALL {
            for derived_dim in DerivedDimension::ALL {
                cells.push((posture, build_dim, derived_dim));
            }
        }
    }
    assert_eq!(cells.len(), 36);

    run_parallel_items(
        "C4b.3 Host matrix",
        &cells,
        |&(posture, build_dim, derived_dim)| {
            let w_req = opening::serial();
            let w_exec = ExecutionRequest::serial(&w_req);
            let seam_spec = format!("mutate:{}:after_wal", posture.ordinal(ordinals));
            let label = format!("matrix/{posture:?}/{build_dim:?}/{derived_dim:?}");
            let verdict = run_host_cut_cell(
                (&seam_spec, &label),
                (build_dim, derived_dim),
                &reference,
                &w_exec,
            );
            posture.assert_matches(&verdict, &label);
        },
    );
}

#[test]
fn host_all_store_seams_across_build_and_d14_discard() {
    run_all_store_seams_exhaustive();
}

pub(super) fn run_all_store_seams_exhaustive() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let probe = RestorableHome::prepare(|loc| prepare_prefix(loc, &execution));
    let (mutate_appends, checkpoint_appends, close_appends) = (
        measure_appends(&probe, "count_mutate"),
        measure_appends(&probe, "count_checkpoint"),
        measure_appends(&probe, "count_close"),
    );
    assert!(
        mutate_appends >= 3,
        "seam-dims/count_mutate: expected >= 3 appends, got {mutate_appends}"
    );
    assert!(
        checkpoint_appends >= 1,
        "seam-dims/count_checkpoint: expected >= 1 appends, got {checkpoint_appends}"
    );
    assert!(
        close_appends >= 1,
        "seam-dims/count_close: expected >= 1 appends, got {close_appends}"
    );

    let mutate_modes = [
        DerivedDimension::DiscardedBeforeKill,
        DerivedDimension::DiscardedBetweenReopens,
        DerivedDimension::DiscardedBeforeClose,
    ];
    let cp_close_modes = [
        DerivedDimension::DiscardedBetweenReopens,
        DerivedDimension::DiscardedBeforeClose,
    ];
    let mut cuts = Vec::new();
    for ord in 1..=mutate_appends {
        for (idx, (seam, _)) in ALL_SEAMS.iter().enumerate() {
            cuts.push((
                "mutate",
                ord,
                *seam,
                mutate_modes[(ord + idx) % mutate_modes.len()],
            ));
        }
    }
    for (op, count) in [("checkpoint", checkpoint_appends), ("close", close_appends)] {
        for ord in 1..=count {
            for (idx, (seam, _)) in ALL_SEAMS.iter().enumerate() {
                cuts.push((
                    op,
                    ord,
                    *seam,
                    cp_close_modes[(ord + idx) % cp_close_modes.len()],
                ));
            }
        }
    }

    let seen_mutate = std::sync::Mutex::new(BTreeSet::new());
    let seen_ref = &seen_mutate;
    run_parallel_items(
        "C4b.3 Store seams x dims",
        &cuts,
        |&(op, ord, seam, derived_dim)| {
            let w_req = opening::serial();
            let w_exec = ExecutionRequest::serial(&w_req);
            let spec = format!("{op}:{ord}:{seam}");
            let label = format!("seam-dims/{spec}/{derived_dim:?}");
            let verdict = run_host_cut_cell(
                (&spec, &label),
                (BuildDimension::Alternating, derived_dim),
                &reference,
                &w_exec,
            );
            match op {
                "mutate" => {
                    let tag = match verdict {
                        RecoveredPrefix::PreOperation => "pre",
                        RecoveredPrefix::UnpublishedCustody { .. } => "custody",
                        RecoveredPrefix::CommittedInFlight => "committed",
                    };
                    seen_ref.lock().unwrap().insert(tag);
                }
                "checkpoint" => assert_eq!(verdict, RecoveredPrefix::PreOperation, "{label}"),
                "close" => assert_eq!(verdict, RecoveredPrefix::CommittedInFlight, "{label}"),
                other => panic!("unknown seam op {other}"),
            }
        },
    );
    assert_eq!(
        seen_mutate.into_inner().unwrap(),
        BTreeSet::from(["pre", "custody", "committed"]),
        "exhaustive Store-seam sweep across build change + D14 discard must cover all 3 postures"
    );
}

fn run_host_cut_cell(
    cut: (&str, &str),
    dims: (BuildDimension, DerivedDimension),
    reference: &ReferenceSnapshots,
    execution: &ExecutionRequest<'_, '_>,
) -> RecoveredPrefix {
    let (seam_spec, label) = cut;
    let (build_dim, derived_dim) = dims;
    let root = tempfile::tempdir().unwrap();
    let home_dir = root.path().join("home");
    let builds = build_dim.sequence();
    let mut seq = 0usize;
    let mut next_chan = |tag: &str| {
        seq += 1;
        root.path().join(format!("ch-{seq}-{tag}"))
    };

    prepare_prefix(&home_dir, execution);
    let (seeded, mut active_discard_gen) = if derived_dim == DerivedDimension::DiscardedBeforeKill {
        let (mut s, opened) =
            Session::start_with_build(&home_dir, &next_chan("pre-discard"), 0, Some(BUILD_PRIMARY));
        assert_eq!(
            (opened.initialized, opened.contacts),
            (0, 0),
            "{label}: pre-discard open"
        );
        let discard_gen = checkpoint_and_discard(&mut s);
        close_expecting_discarded(&mut s, label);
        (None, Some(discard_gen))
    } else {
        (
            Some(capture_seeded_coverage(
                &home_dir,
                &next_chan("seeded"),
                label,
            )),
            None,
        )
    };

    kill_at_seam_with_ctx(
        &home_dir,
        seam_spec,
        TARGET_KEY,
        TARGET_VAL,
        BUILD_PRIMARY,
        label,
    );

    let (mut session, opening1) =
        Session::start_with_build(&home_dir, &next_chan("reopen1"), 0, Some(builds[0]));
    assert_eq!(
        (opening1.initialized, opening1.contacts),
        (0, 0),
        "{label}: reopen#1"
    );
    let cadence1 = inspect_cadence(&mut session);
    if derived_dim == DerivedDimension::DiscardedBeforeKill {
        assert_cadence_discarded(&cadence1, &format!("{label}/reopen1"));
    } else {
        assert_cadence_retained(&cadence1, seeded.unwrap(), &format!("{label}/reopen1"));
    }
    let mut map = HostOracleComparison::default();
    let reopened1 = opening1.state.expect("reopen#1 state");
    let verdict1 =
        classify_and_verify_reopen(&mut map, &reopened1, &reference.before, &reference.after)
            .unwrap_or_else(|e| panic!("{label}: reopen#1 divergence: {e:?}"));

    let (reopened, verdict) = if derived_dim == DerivedDimension::DiscardedBetweenReopens {
        active_discard_gen = Some(checkpoint_and_discard(&mut session));
        session.kill_live_with_ctx(&format!("{label}/d14-kill"));
        let (next_session, opening2) =
            Session::start_with_build(&home_dir, &next_chan("reopen2"), 0, Some(builds[1]));
        session = next_session;
        assert_eq!(
            (opening2.initialized, opening2.contacts),
            (0, 0),
            "{label}: reopen#2"
        );
        assert_cadence_discarded(&inspect_cadence(&mut session), &format!("{label}/reopen2"));
        let reopened2 = opening2.state.expect("reopen#2 state");
        let verdict2 =
            classify_and_verify_reopen(&mut map, &reopened2, &reference.before, &reference.after)
                .unwrap_or_else(|e| panic!("{label}: reopen#2 divergence: {e:?}"));
        assert_eq!(
            verdict2, verdict1,
            "{label}: custody/verdict must stay identical across D14 discard"
        );
        assert_eq!(
            reopened2.head, reopened1.head,
            "{label}: head must stay identical across D14 discard"
        );
        assert_same_custody_rows(&reopened1.custody, &reopened2.custody, label);
        (reopened2, verdict2)
    } else {
        (reopened1, verdict1)
    };

    settle_drift_and_follow(
        &mut session,
        &mut map,
        &verdict,
        &reopened,
        reference,
        label,
    );
    finish_cell_builds(
        (&home_dir, &mut next_chan),
        (session, derived_dim, builds, active_discard_gen),
        (&mut map, reference, label),
    );
    verdict
}
