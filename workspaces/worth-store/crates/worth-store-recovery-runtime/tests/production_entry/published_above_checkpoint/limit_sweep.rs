//! A recovery admitted under less than its world needs is told which limit
//! ran out and the value it was admitted under, at every limit below the
//! need and whichever phase met it. None of them is told its media is
//! damaged. The same media answers every probe: a blocked recovery has no
//! effect.
//!
//! Manifest entries, manifest bytes, observation bytes and staging bytes
//! are each swept from one up to the need.

use super::super::*;
use super::manifest_entry_limit::{released_world, IDLE_NEED, ORDERED_NEED, RELEASED_NEED};
use entries::{entry_need, History, Need, Staging};
use pending_wal_world::{PendingWalWorld, Tail, Workload};
use worlds::Killed;
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlock, PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension,
    PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome, WorthStoreRecovery,
};

#[path = "limit_sweep/entries.rs"]
mod entries;
#[path = "limit_sweep/moved_media.rs"]
mod moved_media;
#[path = "limit_sweep/named.rs"]
mod named;
#[path = "limit_sweep/worlds.rs"]
mod worlds;

/// What the first reopen needs where a checkpoint heads the released
/// object: the release gate then replays the head path its roster holds.
/// Every root read costs one entry, found or not. Among them: the successor
/// root the candidate probe finds absent (1), and the historical roots a
/// blob record is looked up under (4).
const HEADED_NEED: u64 = 95;

/// A certified release reopened with nothing pending: the gate replays its
/// checkpoint head and nothing after it.
const CERTIFIED_NEED: u64 = 39;

/// The released world once its successor was sealed and served: the walk
/// orders that completed history above the same checkpoint.
const COMPLETED_NEED: u64 = 232;

/// A tier release killed after its descriptor WAL: the gate reads the
/// released directory's records and redoes the pending release.
const TIER_NEED: u64 = 50;

/// One checkpointed record rewritten inline: completing it rereads the one
/// moved segment.
const REWRITE_NEED: u64 = 6;

/// One extent copied to another arena: completing it rereads the copy's
/// chunks.
const COPY_NEED: u64 = 11;

/// The entries the two-chunk worlds admit.
const SUFFICIENT_ENTRIES: u64 = 4096;

/// How far a byte sweep moves past a block that names the limit without
/// the count its read reached. A page admission rereads whole pages of
/// these worlds; discovery and the phases after the walk read records.
const PAGE_STRIDE: u64 = 8192;
const RECORD_STRIDE: u64 = 128;

#[derive(Clone, Copy, Debug)]
enum Swept {
    ManifestEntries,
    ManifestBytes,
    ObservationBytes,
    StagingBytes,
}

impl Swept {
    fn request(self, root: &Path, admitted: u64) -> PhysicalRecoveryOpenRequest {
        certified_release_serving::request_narrowing(root, |declared| match self {
            Self::ManifestEntries => declared.manifest_entries = admitted,
            Self::ManifestBytes => declared.manifest_bytes = admitted,
            Self::ObservationBytes => declared.observation_bytes = admitted,
            Self::StagingBytes => declared.staging_bytes = admitted,
        })
    }

    fn dimension(self) -> PhysicalRecoveryLimitDimension {
        match self {
            Self::ManifestEntries => PhysicalRecoveryLimitDimension::ManifestEntries,
            Self::ManifestBytes => PhysicalRecoveryLimitDimension::ManifestBytes,
            Self::ObservationBytes => PhysicalRecoveryLimitDimension::ObservationBytes,
            Self::StagingBytes => PhysicalRecoveryLimitDimension::StagingBytes,
        }
    }
}

fn idle_world() -> PendingWalWorld {
    pending_wal_world::published_above_checkpoint(Workload::ThreeSmallObjects, Tail::Idle)
}

/// Whether this block says anything but that a limit ran out. Discovery,
/// which counts no planning, keeps the reader failure that met the limit as
/// the source of its block; planning names the limit in its denial, or
/// names nothing.
fn names_damage(blocked: &PhysicalRecoveryBlock) -> bool {
    let evidence = blocked.evidence();
    if blocked.cause().limit().is_some() && evidence.planning_counters.is_none() {
        return evidence.planning_denial.is_some();
    }
    !evidence.source_denials.is_empty()
        || evidence
            .planning_denial
            .as_ref()
            .is_some_and(|denial| !named::only_a_limit(denial))
}

/// How this block falls short of naming the swept limit, if it does.
fn shortfall(blocked: &PhysicalRecoveryBlock, swept: Swept, admitted: u64) -> Option<String> {
    let evidence = blocked.evidence();
    let named_limit = blocked.cause().limit().is_some_and(|limit| {
        limit.dimension() == swept.dimension()
            && limit.admitted() == admitted
            && limit.observed() > admitted
    });
    (blocked.recovery_effects() != 0 || names_damage(blocked) || !named_limit).then(|| {
        format!(
            "kind={:?} denial={:?} limit={:?} sources={:?} effects={}",
            blocked.cause(),
            evidence.planning_denial,
            blocked.cause().limit(),
            evidence.source_denials,
            blocked.recovery_effects(),
        )
    })
}

/// Whether page admission blocked, at a limit or not.
fn paged(blocked: &PhysicalRecoveryBlock) -> bool {
    blocked.cause().phase() == PhysicalRecoveryBlockKind::PageAdmission
}

/// What a probe that did not recover was told.
struct Blocked {
    /// Whether page admission blocked.
    paged: bool,
    /// The count the block says its charge or read reached, or one past the
    /// limit where it names none.
    count: u64,
    /// The phase that blocked and the outermost denial it named.
    denial: String,
}

/// What the probe under `admitted` was told; `None` once the world
/// recovers. A block that does not name the limit is recorded against the
/// limit it was admitted under.
fn reached(
    root: &Path,
    swept: Swept,
    admitted: u64,
    failures: &mut Vec<String>,
) -> Option<Blocked> {
    match WorthStoreRecovery::recover(swept.request(root, admitted)) {
        PhysicalRecoveryOutcome::Recovered(handoff) => {
            drop(handoff);
            None
        }
        PhysicalRecoveryOutcome::Blocked(blocked) => {
            if let Some(short) = shortfall(&blocked, swept, admitted) {
                failures.push(format!("{swept:?} {admitted}: {short}"));
            }
            let count = blocked.cause().limit().map_or(0, |limit| limit.observed());
            let denial = format!("{:?}", blocked.evidence().planning_denial);
            let outermost = denial
                .split(['(', ' '])
                .take(2)
                .collect::<Vec<_>>()
                .join("");
            Some(Blocked {
                paged: paged(&blocked),
                count: count.max(admitted + 1),
                denial: format!("{:?} {outermost}", blocked.cause().phase()),
            })
        }
        outcome => panic!("{swept:?} {admitted}: neither recovered nor blocked: {outcome:?}"),
    }
}

/// Byte limits from one byte up to the one the world recovers under, which
/// is answered. A block that names the count its read reached is followed
/// by a recovery admitted exactly that count, so every such read is refused
/// once. The others are passed a stride at a time: a page where ordering
/// the history blocked, a record elsewhere.
fn byte_need(root: &Path, swept: Swept, failures: &mut Vec<String>) -> Option<u64> {
    let mut admitted = 1;
    while admitted <= certified_release_serving::ADMITTED_BYTES {
        let Some(Blocked { paged, count, .. }) = reached(root, swept, admitted, failures) else {
            return Some(admitted);
        };
        admitted = if count > admitted + 1 {
            count
        } else if paged {
            admitted + PAGE_STRIDE
        } else {
            admitted + RECORD_STRIDE
        };
    }
    None
}

/// Each sweep takes the world as the kill left it. All run before any is
/// judged, so one report lists every limit that was not named. The entry
/// need is fixed; bytes follow block sizes, so those needs are only found.
fn assert_limits_are_reported_as_limits<W: Killed>(
    killed: impl Fn() -> W,
    need: Need,
    stage: &str,
) {
    let mut failures = Vec::new();
    let entries = entry_need(&killed, need, &mut failures);
    let manifest = byte_need(killed().root(), Swept::ManifestBytes, &mut failures);
    let bytes = byte_need(killed().root(), Swept::ObservationBytes, &mut failures);
    let staging = byte_need(killed().root(), Swept::StagingBytes, &mut failures);
    assert!(
        failures.is_empty(),
        "{stage}: {} blocks under {entries:?} entries, {manifest:?} manifest bytes, \
         {bytes:?} observation bytes and {staging:?} staging bytes did not name the limit:\n{}",
        failures.len(),
        failures.join("\n"),
    );
    assert_eq!(
        entries,
        Some(need.entries),
        "{stage}: the entry need is fixed"
    );
    assert!(
        manifest.is_some_and(|bytes| bytes > RECORD_STRIDE),
        "{stage}: no manifest-byte limit blocked: {manifest:?}",
    );
    assert!(
        bytes.is_some_and(|bytes| bytes > PAGE_STRIDE),
        "{stage}: no observation-byte limit blocked: {bytes:?}",
    );
    match need.staging {
        Staging::Staged => assert!(
            staging.is_some_and(|bytes| bytes > PAGE_STRIDE),
            "{stage}: no staging-byte limit blocked: {staging:?}",
        ),
        Staging::Unstaged => assert_eq!(staging, Some(1), "{stage}: nothing is staged"),
    }
}

#[test]
fn every_limit_under_the_need_of_published_objects_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        idle_world,
        Need::every(IDLE_NEED, History::Ordered),
        "published above the checkpoint",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II M12/M15"]
fn every_limit_under_the_need_of_a_historical_release_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        released_world,
        Need::every(RELEASED_NEED, History::Ordered),
        "released above the checkpoint",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II M12/M15"]
fn every_limit_under_the_need_of_a_release_above_a_checkpoint_head_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        pending_successor_above_history::terminal_successor_of_a_checkpoint_head,
        Need::every(HEADED_NEED, History::Ordered),
        "released above a checkpoint head",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II M12/M15"]
fn every_limit_under_the_need_of_a_certified_release_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        worlds::certified_release,
        Need::every(CERTIFIED_NEED, History::Ordered).unstaged(),
        "certified release",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II M12/M15"]
fn every_limit_under_the_need_of_completed_history_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        worlds::completed_history,
        Need::strided(COMPLETED_NEED, 16),
        "completed history",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II M12/M15"]
fn every_limit_under_the_need_of_a_pending_tier_release_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        tier_release_pending_wal::kill_producer_after_descriptor_wal,
        Need::every(TIER_NEED, History::Unordered),
        "pending tier release",
    );
}

#[test]
fn every_limit_under_the_need_of_a_killed_rewrite_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        moved_media::killed_rewrite,
        Need::every(REWRITE_NEED, History::Unordered),
        "killed rewrite",
    );
}

#[test]
fn every_limit_under_the_need_of_a_killed_copy_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        moved_media::killed_copy,
        Need::every(COPY_NEED, History::Ordered),
        "killed copy",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II M12/M15"]
fn every_limit_under_the_need_of_a_pending_successor_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        pending_successor_above_history::ordered_release_above_a_head_checkpoint,
        Need::strided(ORDERED_NEED, 16),
        "pending successor of an ordered release",
    );
}
