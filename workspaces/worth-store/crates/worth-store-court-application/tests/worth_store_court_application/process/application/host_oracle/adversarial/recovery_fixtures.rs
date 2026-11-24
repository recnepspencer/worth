//! C4b.2: Unclean Query Host Home fixture preparation for recovery-interruption
//! courts (CommittedInFlight, UnpublishedCustody, PreOperation cold-release,
//! CleanupWalSegments with small certification segments, and
//! IndeterminateReplacement for ObligationDischarge).
use super::super::opening;
use super::oracle_verifier::{
    build_reference_with_prefix, open_dir, prepare_prefix, ReferenceSnapshots, PREFIX_KEY,
    PREFIX_VAL, TARGET_KEY, TARGET_VAL,
};
use super::seam_kill_sweep::kill_at_seam;
use super::RestorableHome;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::{atomic::AtomicUsize, Arc};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution;
use worth_store::physical_runtime::{
    certification::{MediaFaultDirective, MediaOperationRole},
    FilesystemAccessPosture, FilesystemMediaAdmission,
};
use worth_store_application::ApplicationHomeCertificationProfile;
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement, host_observation::requests,
};

pub(super) const CLEANUP_WAL_BYTES: u64 = 2 << 20;

const FIXTURE_CHILD: &str =
    "process::application::host_oracle::adversarial::recovery_fixtures::host_recovery_fixture_child";
const ENV_HOME: &str = "WORTH_ADV_REC_FIX_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_REC_FIX_MARKERS";
const ENV_MODE: &str = "WORTH_ADV_REC_FIX_MODE";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MutateSeamOrdinals {
    pub pre_op_after_wal: usize,
    pub custody_after_wal: usize,
    pub committed_after_wal: usize,
}

/// Discover exact physical append ordinals (killed at `after_wal` so Store
/// physical recovery must execute full WAL redo!) for PreOperation (after
/// OpenAttempt), UnpublishedCustody (after Native), and CommittedInFlight
/// (after World WAL).
pub(super) fn discover_mutate_ordinals(execution: &ExecutionRequest<'_, '_>) -> MutateSeamOrdinals {
    let probe = RestorableHome::prepare(|loc| prepare_prefix(loc, execution));
    let mut pre_op = None;
    let mut custody = None;
    let mut committed = None;
    for ordinal in 1..=8 {
        probe.restore();
        kill_at_seam(probe.location(), &format!("mutate:{ordinal}:after_wal"));
        match probe_resolution(probe.location(), execution) {
            Resolution::Unseen if pre_op.is_none() => pre_op = Some(ordinal),
            Resolution::ProductUnpublished(_) if custody.is_none() => custody = Some(ordinal),
            Resolution::RecoveredCommitted(_) => {
                committed = Some(ordinal);
                break;
            }
            _ => {}
        }
    }
    MutateSeamOrdinals {
        pre_op_after_wal: pre_op.expect("mutate must have a PreOperation WAL cut"),
        custody_after_wal: custody.expect("mutate must have a ProductUnpublished WAL cut"),
        committed_after_wal: committed.expect("mutate must have a CommittedInFlight WAL cut"),
    }
}

fn probe_resolution(
    location: &Path,
    execution: &ExecutionRequest<'_, '_>,
) -> Resolution {
    let (home, _) = open_dir(location, None, Arc::new(AtomicUsize::new(0)), execution);
    let resolution = requests::resolve_original(home.runtime(), TARGET_KEY, TARGET_VAL).unwrap();
    drop(home);
    resolution
}

pub(super) fn prepare_prefix_unclosed(location: &Path, execution: &ExecutionRequest<'_, '_>) {
    let (home, _) = open_dir(location, None, Arc::new(AtomicUsize::new(0)), execution);
    assert!(requests::set(home.runtime(), PREFIX_KEY, PREFIX_VAL).receipt().is_some());
    drop(home);
}

pub(super) fn latest_root_counts(location: &Path) -> (u64, u16) {
    let mut best_gen = 0u64;
    let mut best = (0u64, 0u16);
    let mut stack = vec![location.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(bytes) = std::fs::read(&path)
                && let Ok((m, _)) =
                    worth_store_physical_format::DurablePhysicalRootManifest::decode(&bytes, u16::MAX)
                && m.generation() > best_gen
            {
                best_gen = m.generation();
                best = (m.record_count(), m.node_capacity());
            }
        }
    }
    assert!(best_gen > 0, "store must contain a decoded RootManifest");
    best
}

/// Prepare an unclean `committed` fixture whose recovered root manifest sits
/// strictly below one leaf (`record_count < node_capacity`), making opening
/// sync counts 100% deterministic (non-splitting layout).
pub(super) fn prepare_non_splitting_posture(
    posture: &str,
    ordinals: MutateSeamOrdinals,
    execution: &ExecutionRequest<'_, '_>,
) -> RestorableHome {
    let target = match posture {
        "committed" => ordinals.committed_after_wal,
        "custody" => ordinals.custody_after_wal,
        "preop" => ordinals.pre_op_after_wal,
        other => panic!("unknown posture {other}"),
    };
    let home = RestorableHome::prepare(|loc| {
        prepare_prefix_unclosed(loc, execution);
        kill_at_seam(loc, &format!("mutate:{target}:after_wal"));
    });
    let (rc, cap) = latest_root_counts(home.location());
    assert!(rc + 8 < u64::from(cap), "{posture} non-splitting fixture must stay within 1 leaf (rc={rc}, cap={cap})");
    home
}

/// Prepare an unclean `committed` fixture whose post-physical-recovery root has
/// `record_count == node_capacity` (one 100%-full leaf), so `DescriptorIncarnation`
/// deterministically splits the leaf on EVERY reopen regardless of random epoch UUID.
pub(super) fn prepare_guaranteed_split_committed(
    ordinals: MutateSeamOrdinals,
    execution: &ExecutionRequest<'_, '_>,
) -> (RestorableHome, ReferenceSnapshots) {
    let mut probe_pre_kill_rc = 0u64;
    let probe = RestorableHome::prepare(|loc| {
        prepare_prefix_unclosed(loc, execution);
        let (h0, _) = open_dir(loc, None, Arc::new(AtomicUsize::new(0)), execution);
        drop(h0);
        probe_pre_kill_rc = latest_root_counts(loc).0;
        kill_at_seam(loc, &format!("mutate:{}:after_wal", ordinals.committed_after_wal));
    });
    let (before_rc, cap) = latest_root_counts(probe.location());
    let cap_u64 = u64::from(cap);
    let kill_delta = before_rc - probe_pre_kill_rc;
    // Count how many records a single reopen+drop (DescriptorIncarnation) adds.
    let (home_p, _) = open_dir(probe.location(), None, Arc::new(AtomicUsize::new(0)), execution);
    drop(home_p);
    let (after_one_open, _) = latest_root_counts(probe.location());
    assert!(after_one_open > before_rc, "reopen must add at least 1 record");
    let (home_p2, _) = open_dir(probe.location(), None, Arc::new(AtomicUsize::new(0)), execution);
    drop(home_p2);
    let (after_two_opens, _) = latest_root_counts(probe.location());
    let per_open = after_two_opens - after_one_open;
    let phys_delta = after_one_open - before_rc - per_open;
    assert_eq!(per_open, 1, "DescriptorIncarnation must add exactly 1 record per reopen");

    let target_before_kill = cap_u64 - phys_delta - kill_delta;
    let mut prefix = vec![(PREFIX_KEY, PREFIX_VAL)];
    let home = RestorableHome::prepare(|loc| {
        prepare_prefix_unclosed(loc, execution);
        let mut idx = 1u64;
        while latest_root_counts(loc).0 + 8 <= target_before_kill {
            let (h, _) = open_dir(loc, None, Arc::new(AtomicUsize::new(0)), execution);
            let k = PREFIX_KEY + 0x200 + idx;
            let v = PREFIX_VAL + idx;
            assert!(requests::set(h.runtime(), k, v).receipt().is_some());
            prefix.push((k, v));
            drop(h);
            idx += 1;
        }
        while latest_root_counts(loc).0 < target_before_kill {
            let (h, _) = open_dir(loc, None, Arc::new(AtomicUsize::new(0)), execution);
            drop(h);
        }
        assert_eq!(latest_root_counts(loc).0, target_before_kill);
        kill_at_seam(loc, &format!("mutate:{}:after_wal", ordinals.committed_after_wal));
    });
    // Verify post-physical-recovery record_count is EXACTLY `cap` (every leaf 100% full).
    let (pre_open_rc, _) = latest_root_counts(home.location());
    assert_eq!(pre_open_rc + phys_delta, cap_u64, "split fixture must fill leaf to exact capacity {cap_u64}");
    let reference = build_reference_with_prefix(&prefix, execution);
    (home, reference)
}

pub(super) fn prepare_committed_in_flight(
    ordinals: MutateSeamOrdinals,
    execution: &ExecutionRequest<'_, '_>,
) -> RestorableHome {
    RestorableHome::prepare(|loc| {
        prepare_prefix(loc, execution);
        kill_at_seam(loc, &format!("mutate:{}:after_wal", ordinals.committed_after_wal));
    })
}

pub(super) fn prepare_unpublished_custody(
    ordinals: MutateSeamOrdinals,
    execution: &ExecutionRequest<'_, '_>,
) -> RestorableHome {
    RestorableHome::prepare(|loc| {
        prepare_prefix(loc, execution);
        kill_at_seam(loc, &format!("mutate:{}:after_wal", ordinals.custody_after_wal));
    })
}

pub(super) fn prepare_pre_operation(
    ordinals: MutateSeamOrdinals,
    execution: &ExecutionRequest<'_, '_>,
) -> RestorableHome {
    RestorableHome::prepare(|loc| {
        prepare_prefix(loc, execution);
        kill_at_seam(loc, &format!("mutate:{}:after_wal", ordinals.pre_op_after_wal));
    })
}

/// Prepare a Host Home with small WAL segments, multi-segment prefix writes,
/// and a published checkpoint killed before WAL reclamation so recovery must
/// reach `CleanupFreshnessRead` and `CleanupRemoval`.
pub(super) fn prepare_cleanup_wal_fixture(
    execution: &ExecutionRequest<'_, '_>,
) -> (RestorableHome, ReferenceSnapshots) {
    let mut prefix_count = 0usize;
    let home = RestorableHome::prepare(|loc| {
        let note = run_fixture_child(loc, "cleanup_checkpoint");
        prefix_count = note.trim().parse().expect("cleanup child must report prefix mutation count");
    });
    assert!(prefix_count >= 2, "cleanup fixture must write multiple prefix commits");
    let prefix = cleanup_prefix_pairs(prefix_count);
    let reference = build_reference_with_prefix(&prefix, execution);
    (home, reference)
}

/// Prepare a Host Home carrying an indeterminate root replacement so recovery
/// must reach `ObligationDischarge`.
pub(super) fn prepare_indeterminate_replacement(
    ordinals: MutateSeamOrdinals,
    execution: &ExecutionRequest<'_, '_>,
) -> RestorableHome {
    RestorableHome::prepare(|loc| {
        prepare_prefix(loc, execution);
        kill_at_seam(loc, &format!("mutate:{}:after_wal", ordinals.committed_after_wal));
        let _ = run_fixture_child(loc, "indeterminate_replace");
    })
}

fn cleanup_prefix_pairs(count: usize) -> Vec<(u64, u64)> {
    let mut pairs = Vec::with_capacity(count);
    for i in 0..count as u64 {
        let key = if i == 0 { PREFIX_KEY } else { PREFIX_KEY + 0x100 + i };
        pairs.push((key, PREFIX_VAL + i));
    }
    pairs
}

fn count_wal_segments(root: &Path) -> usize {
    let mut stack = vec![root.to_path_buf()];
    let mut count = 0;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if dir.file_name().and_then(|n| n.to_str()) == Some("wal") {
                count += 1;
            }
        }
    }
    count
}

fn run_fixture_child(location: &Path, mode: &str) -> String {
    let markers = tempfile::tempdir().unwrap();
    let mode_path = PathBuf::from(mode);
    let ready = markers.path().join("ready");
    let end = crate::killed_child::run_and_kill(
        FIXTURE_CHILD,
        &[
            (ENV_HOME, location),
            (ENV_MARKERS, markers.path()),
            (ENV_MODE, &mode_path),
        ],
        &ready,
        &markers.path().join("finished"),
    );
    assert_eq!(
        end,
        crate::killed_child::ChildEnd::KilledAtPark,
        "fixture child {mode} must park and be killed"
    );
    std::fs::read_to_string(&ready).unwrap_or_default()
}

#[test]
fn host_recovery_fixture_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let mode = std::env::var(ENV_MODE).unwrap();
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    match mode.as_str() {
        "cleanup_checkpoint" => {
            let profile = ApplicationHomeCertificationProfile::with_wal_segment_bytes(
                NonZeroU64::new(CLEANUP_WAL_BYTES).unwrap(),
            );
            profile.run(|| {
                let media = FilesystemMediaAdmission::certification(
                    FilesystemAccessPosture::CoordinatedServiceAccount,
                );
                let authority = media.fault_schedule_authority();
                let activation = authority.one_shot_activation();
                let rule = authority
                    .rule(
                        MediaOperationRole::Delete,
                        64,
                        MediaFaultDirective::FailBefore {
                            kind: std::io::ErrorKind::Other,
                            raw_os_error: None,
                        },
                    )
                    .for_nth_identified_operation_after_activation(
                        activation.clone(),
                        NonZeroU64::MIN,
                    );
                let schedule = authority.schedule(vec![rule]).unwrap();
                let (home, _) = open_dir(
                    &location,
                    Some(AdversarialPlacement::new().with_media(media.with_fault_schedule(schedule))),
                    Arc::new(AtomicUsize::new(0)),
                    &execution,
                );
                let mut written = 0usize;
                for i in 0..256u64 {
                    let key = if i == 0 { PREFIX_KEY } else { PREFIX_KEY + 0x100 + i };
                    assert!(requests::set(home.runtime(), key, PREFIX_VAL + i).receipt().is_some());
                    written += 1;
                    if written >= 2 && count_wal_segments(&location) >= 2 {
                        break;
                    }
                }
                assert!(count_wal_segments(&location) >= 2, "cleanup fixture must roll >= 2 WAL segments");
                activation.arm().unwrap();
                home.checkpoint(&execution)
                    .unwrap_or_else(|e| panic!("cleanup checkpoint must publish: {e:?}"));
                assert!(
                    count_wal_segments(&location) >= 2,
                    "cleanup fixture must retain covered WAL segments after checkpoint (deletes={})",
                    activation.matching_operation_count()
                );
                crate::killed_child::announce(&markers.join("ready"), written.to_string().as_bytes());
                crate::killed_child::park();
            });
        }
        "indeterminate_replace" => {
            let media = FilesystemMediaAdmission::certification(
                FilesystemAccessPosture::CoordinatedServiceAccount,
            );
            let authority = media.fault_schedule_authority();
            let activation = authority.one_shot_activation();
            let rule = authority
                .rule(
                    MediaOperationRole::AtomicReplace,
                    1,
                    MediaFaultDirective::IndeterminateAfterEffect,
                )
                .for_nth_identified_operation_after_activation(
                    activation.clone(),
                    NonZeroU64::MIN,
                );
            let schedule = authority.schedule(vec![rule]).unwrap();
            let (home, _) = open_dir(
                &location,
                Some(AdversarialPlacement::new().with_media(media.with_fault_schedule(schedule))),
                Arc::new(AtomicUsize::new(0)),
                &execution,
            );
            activation.arm().unwrap();
            assert!(home.checkpoint(&execution).is_err(), "indeterminate replace must fail checkpoint");
            assert!(activation.is_consumed(), "AtomicReplace fault must fire");
            crate::killed_child::announce(&markers.join("ready"), b"indeterminate");
            crate::killed_child::park();
        }
        other => panic!("unknown recovery fixture mode {other}"),
    }
}
