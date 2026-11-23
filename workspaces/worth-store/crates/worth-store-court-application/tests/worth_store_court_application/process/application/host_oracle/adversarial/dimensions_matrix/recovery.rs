//! C4b.3: Recovery interruption at EVERY physical recovery stage (all 13),
//! EVERY ApplicationHomeOpeningStage edge (all 12), AND 4-deep chained kills
//! across build-identity change (all 3 alternating builds) and D14 discard —
//! every single reopen verified in a genuinely fresh OS `Session` process.
use super::super::super::{opening, session::Session};
use super::super::oracle_verifier::{build_reference, prepare_prefix, ReferenceSnapshots};
use super::super::recovery_fixtures::{
    discover_mutate_ordinals, prepare_cleanup_wal_fixture, prepare_indeterminate_replacement,
    CLEANUP_WAL_BYTES,
};
use super::super::recovery_interruption_oracle::{
    kill_recovery_with_build, opening_stage_index, PHYSICAL_STAGES,
};
use super::super::seam_kill_sweep::kill_at_seam;
use super::super::{BUILD_PRIMARY, BUILD_SECONDARY, BUILD_TERTIARY};
use super::verify::{
    assert_same_custody_rows, checkpoint_and_discard, close_expecting_discarded, close_normal,
    inspect_cadence, settle_drift_and_follow,
};
use super::{assert_cadence_checkpointed, assert_cadence_discarded, run_parallel_items, Posture};
use std::path::Path;
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::PhysicalRecoveryYieldpointStage as PhysStage;
use worth_store_application::{
    ApplicationHomeOpeningEdge as OpenEdge, ApplicationHomeOpeningStage as OpenStage,
};
use worth_store_testkit_application_oracle::facade::host_observation::{
    adversarial_verdict::classify_and_verify_reopen,
    comparison::{self, HostOracleComparison},
};

#[test]
fn host_recovery_every_stage_across_build_change_and_d14_discard() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let ordinals = discover_mutate_ordinals(&execution);
    let (cleanup, cleanup_ref) = prepare_cleanup_wal_fixture(&execution);
    let indeterminate = prepare_indeterminate_replacement(ordinals, &execution);

    // Part A: ALL 13 PhysicalRecoveryYieldpointStages across fresh OS processes,
    // all 3 alternating builds, and D14 discard.
    let mut standard_phys = Vec::new();
    for stage in PHYSICAL_STAGES {
        let spec = format!("phys:{}", stage.label());
        match stage {
            PhysStage::CleanupFreshnessRead | PhysStage::CleanupRemoval => {
                cleanup.restore();
                kill_recovery_with_build(
                    cleanup.location(),
                    &spec,
                    Some(CLEANUP_WAL_BYTES),
                    Some(BUILD_SECONDARY),
                );
                verify_special_stage_with_d14(
                    cleanup.location(),
                    Some(CLEANUP_WAL_BYTES),
                    Posture::PreOp,
                    &cleanup_ref,
                    &format!("dims/{spec}"),
                );
            }
            PhysStage::ObligationDischarge => {
                indeterminate.restore();
                kill_recovery_with_build(
                    indeterminate.location(),
                    &spec,
                    None,
                    Some(BUILD_SECONDARY),
                );
                verify_special_stage_with_d14(
                    indeterminate.location(),
                    None,
                    Posture::Committed,
                    &reference,
                    &format!("dims/{spec}"),
                );
            }
            _ => standard_phys.push(spec),
        }
    }

    // Part B: ALL 12 ApplicationHomeOpeningStage edges (6 stages x Before/After)
    // plus all 10 standard physical stages on D14-discarded fixtures.
    let mut d14_stage_specs = standard_phys;
    for stage in OpenStage::ALL {
        for edge in [OpenEdge::Before, OpenEdge::After] {
            let edge_tag = if edge == OpenEdge::Before {
                "before"
            } else {
                "after"
            };
            d14_stage_specs.push(format!("open:{}:{edge_tag}", opening_stage_index(stage)));
        }
    }
    assert_eq!(d14_stage_specs.len(), 22);

    run_parallel_items(
        "C4b.3 every recovery stage x D14 x build",
        &d14_stage_specs,
        |spec| {
            let w_req = opening::serial();
            let w_exec = ExecutionRequest::serial(&w_req);
            run_d14_recovery_stage(spec, ordinals.committed_after_wal, &reference, &w_exec);
        },
    );

    // Part C: 4-deep chained recovery kills across all 3 postures + alternating builds + D14 discard.
    let chain = [
        (
            format!("phys:{}", PhysStage::CandidateSynchronization.label()),
            BUILD_SECONDARY,
        ),
        (
            format!("phys:{}", PhysStage::RootProtocolReplacement.label()),
            BUILD_TERTIARY,
        ),
        ("open:2:after".to_owned(), BUILD_PRIMARY),
        ("open:4:after".to_owned(), BUILD_SECONDARY),
    ];
    for posture in Posture::ALL {
        let label = format!("recovery-chain-dims/{posture:?}");
        let root = tempfile::tempdir().unwrap();
        let home_dir = root.path().join("home");
        let discard_gen = prepare_discarded_then_kill(
            &home_dir,
            root.path(),
            posture.ordinal(ordinals),
            &execution,
            &label,
        );
        for (spec, build) in &chain {
            kill_recovery_with_build(&home_dir, spec, None, Some(build));
        }
        verify_recovered_d14_session(
            (&home_dir, root.path()),
            (discard_gen, posture),
            (&reference, &label),
        );
    }
}

/// Verify CleanupFreshnessRead / CleanupRemoval / ObligationDischarge in fresh
/// OS `Session` processes across all 3 alternating builds AND a live D14
/// discard + parent OS kill + fresh reopen asserting `covered == 0`.
fn verify_special_stage_with_d14(
    home_dir: &Path,
    wal_bytes: Option<u64>,
    posture: Posture,
    reference: &ReferenceSnapshots,
    label: &str,
) {
    let ch = tempfile::tempdir().unwrap();
    let (mut s1, o1) = Session::start_with_build_and_wal(
        home_dir,
        &ch.path().join("r1"),
        0,
        Some(BUILD_TERTIARY),
        wal_bytes,
    );
    assert_eq!((o1.initialized, o1.contacts), (0, 0), "{label}: reopen#1");
    let mut map = HostOracleComparison::default();
    for state in &reference.prefix_states {
        map.observe_reference(state).unwrap();
    }
    let reopened1 = o1.state.unwrap();
    let verdict1 =
        classify_and_verify_reopen(&mut map, &reopened1, &reference.before, &reference.after)
            .unwrap_or_else(|e| panic!("{label}: reopen#1 divergence: {e:?}"));
    posture.assert_matches(&verdict1, &format!("{label}/r1"));

    // Cross D14 discard + live OS kill + fresh OS reopen under BUILD_PRIMARY.
    let discard_gen = checkpoint_and_discard(&mut s1);
    s1.kill_live();
    let (mut s2, o2) = Session::start_with_build_and_wal(
        home_dir,
        &ch.path().join("r2"),
        0,
        Some(BUILD_PRIMARY),
        wal_bytes,
    );
    assert_eq!((o2.initialized, o2.contacts), (0, 0), "{label}: reopen#2");
    assert_cadence_discarded(&inspect_cadence(&mut s2), &format!("{label}/r2"));
    let reopened2 = o2.state.unwrap();
    let verdict2 =
        classify_and_verify_reopen(&mut map, &reopened2, &reference.before, &reference.after)
            .unwrap_or_else(|e| panic!("{label}: reopen#2 divergence: {e:?}"));
    assert_eq!(verdict2, verdict1, "{label}: verdict across D14 discard");
    assert_eq!(
        reopened2.head, reopened1.head,
        "{label}: head across D14 discard"
    );
    assert_same_custody_rows(&reopened1.custody, &reopened2.custody, label);

    settle_drift_and_follow(&mut s2, &mut map, &verdict2, &reopened2, reference, label);
    if wal_bytes.is_some() {
        let mid = inspect_cadence(&mut s2);
        assert!(
            mid["relational.canonical-commits"].0 > 0 && mid["world.occurrences"].0 > 0,
            "{label}/auto-cp: automatic cadence checkpoint must resume after D14 discard: {mid:?}"
        );
        close_normal(&mut s2, Some(discard_gen + 1), &format!("{label}/close"));
    } else {
        close_normal(&mut s2, Some(discard_gen), &format!("{label}/close"));
    }

    let (mut s3, o3) = Session::start_with_build_and_wal(
        home_dir,
        &ch.path().join("r3"),
        0,
        Some(BUILD_SECONDARY),
        wal_bytes,
    );
    assert_eq!(
        (o3.initialized, o3.contacts),
        (0, 0),
        "{label}: final reopen"
    );
    assert_cadence_checkpointed(&inspect_cadence(&mut s3), &format!("{label}/final"));
    comparison::compare(&mut map, &o3.state.unwrap(), &reference.after_follow)
        .unwrap_or_else(|e| panic!("{label}: final parity: {e:?}"));
    close_normal(&mut s3, None, &format!("{label}/final-close"));
}

fn prepare_discarded_then_kill(
    home_dir: &Path,
    ch_root: &Path,
    mutate_ordinal: usize,
    execution: &ExecutionRequest<'_, '_>,
    label: &str,
) -> u64 {
    prepare_prefix(home_dir, execution);
    let (mut s0, o0) =
        Session::start_with_build(home_dir, &ch_root.join("ch-0"), 0, Some(BUILD_PRIMARY));
    assert_eq!(
        (o0.initialized, o0.contacts),
        (0, 0),
        "{label}: pre-discard open"
    );
    let discard_gen = checkpoint_and_discard(&mut s0);
    close_expecting_discarded(&mut s0, label);
    kill_at_seam(home_dir, &format!("mutate:{mutate_ordinal}:after_wal"));
    discard_gen
}

fn run_d14_recovery_stage(
    spec: &str,
    committed_ordinal: usize,
    reference: &ReferenceSnapshots,
    execution: &ExecutionRequest<'_, '_>,
) {
    let label = format!("recovery-stage-dims/{spec}");
    let root = tempfile::tempdir().unwrap();
    let home_dir = root.path().join("home");
    let discard_gen =
        prepare_discarded_then_kill(&home_dir, root.path(), committed_ordinal, execution, &label);
    kill_recovery_with_build(&home_dir, spec, None, Some(BUILD_SECONDARY));
    verify_recovered_d14_session(
        (&home_dir, root.path()),
        (discard_gen, Posture::Committed),
        (reference, &label),
    );
}

fn verify_recovered_d14_session(
    paths: (&Path, &Path),
    expected: (u64, Posture),
    check: (&ReferenceSnapshots, &str),
) {
    let (home_dir, ch_root) = paths;
    let (discard_gen, posture) = expected;
    let (reference, label) = check;
    let (mut session, opened) =
        Session::start_with_build(home_dir, &ch_root.join("ch-1"), 0, Some(BUILD_TERTIARY));
    assert_eq!(
        (opened.initialized, opened.contacts),
        (0, 0),
        "{label}: post-kill reopen"
    );
    assert_cadence_discarded(&inspect_cadence(&mut session), &format!("{label}/reopen"));
    let mut map = HostOracleComparison::default();
    let reopened = opened.state.unwrap();
    let verdict =
        classify_and_verify_reopen(&mut map, &reopened, &reference.before, &reference.after)
            .unwrap_or_else(|e| panic!("{label}: post-kill divergence: {e:?}"));
    posture.assert_matches(&verdict, label);
    settle_drift_and_follow(
        &mut session,
        &mut map,
        &verdict,
        &reopened,
        reference,
        label,
    );
    close_normal(&mut session, Some(discard_gen), &format!("{label}/close"));
    let (mut fin, fin_open) =
        Session::start_with_build(home_dir, &ch_root.join("ch-2"), 0, Some(BUILD_PRIMARY));
    assert_eq!(
        (fin_open.initialized, fin_open.contacts),
        (0, 0),
        "{label}: final"
    );
    assert_cadence_checkpointed(&inspect_cadence(&mut fin), &format!("{label}/final"));
    comparison::compare(&mut map, &fin_open.state.unwrap(), &reference.after_follow)
        .unwrap_or_else(|e| panic!("{label}: final parity: {e:?}"));
    close_normal(&mut fin, None, &format!("{label}/final-close"));
}
