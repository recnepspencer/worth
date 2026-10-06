//! A recovery admitted under less than its world needs is told which limit
//! ran out and the value it was admitted under, at every limit below the
//! need and whichever phase met it. None of them is told its media is
//! damaged. The same media answers every probe: a blocked recovery has no
//! effect.
//!
//! Manifest entries, manifest bytes, observation bytes and staging bytes
//! are each swept from one up to the need.

use super::super::*;
use super::manifest_entry_limit::{released_world, IDLE_NEED, RELEASED_NEED};
use pending_wal_world::{PendingWalWorld, Tail, Workload};
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlock, PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension,
    PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome, WorthStoreRecovery,
};

#[path = "limit_sweep/named.rs"]
mod named;

/// What the first reopen needs where a checkpoint heads the released
/// object: the release gate then replays the head path its roster holds.
/// Every root read costs one entry, found or not. Among them: the successor
/// root the candidate probe finds absent (1), and the historical roots a
/// blob record is looked up under (4).
const HEADED_NEED: u64 = 95;

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

/// The count the block says its charge or read reached, or one past the
/// limit where it names none, and whether page admission blocked; `None`
/// once the world recovers. A block that does not name the limit is
/// recorded against the limit it was admitted under.
fn reached(
    world: &PendingWalWorld,
    swept: Swept,
    admitted: u64,
    failures: &mut Vec<String>,
) -> Option<(bool, u64)> {
    match WorthStoreRecovery::recover(swept.request(world.root(), admitted)) {
        PhysicalRecoveryOutcome::Recovered(handoff) => {
            drop(handoff);
            None
        }
        PhysicalRecoveryOutcome::Blocked(blocked) => {
            if let Some(short) = shortfall(&blocked, swept, admitted) {
                failures.push(format!("{swept:?} {admitted}: {short}"));
            }
            let count = blocked.cause().limit().map_or(0, |limit| limit.observed());
            Some((paged(&blocked), count.max(admitted + 1)))
        }
        outcome => panic!("{swept:?} {admitted}: neither recovered nor blocked: {outcome:?}"),
    }
}

/// Every entry limit up to the one the world recovers under, which is
/// answered. A charge is refused at the same count under every limit short
/// of it, and the world recovers under the count its last refusal reached.
/// Ordering the history of every one of these worlds charges some step
/// several entries at once, so one of its refusals names a count further
/// than one past its limit.
/// A recovery changes the world, so the sweep ends there; every block
/// before it left the media as the kill did.
fn entry_need(world: &PendingWalWorld, failures: &mut Vec<String>) -> Option<u64> {
    let mut refused_at = None;
    let mut charged_together = false;
    for admitted in 1..=SUFFICIENT_ENTRIES {
        let blocked = reached(world, Swept::ManifestEntries, admitted, failures);
        let now = blocked.map(|(_, count)| count);
        if let Some(count) = refused_at.filter(|count| admitted < *count && now != Some(*count)) {
            failures.push(format!(
                "ManifestEntries {admitted}: refused at {count} under a lower limit, now {now:?}",
            ));
        }
        let Some((paged, count)) = blocked else {
            if !charged_together {
                failures.push("ManifestEntries: no ordered step named the count it reached".into());
            }
            return Some(admitted);
        };
        charged_together |= paged && count > admitted + 1;
        refused_at = Some(count);
    }
    None
}

/// Byte limits from one byte up to the one the world recovers under, which
/// is answered. A block that names the count its read reached is followed
/// by a recovery admitted exactly that count, so every such read is refused
/// once. The others are passed a stride at a time: a page where ordering
/// the history blocked, a record elsewhere.
fn byte_need(world: &PendingWalWorld, swept: Swept, failures: &mut Vec<String>) -> Option<u64> {
    let mut admitted = 1;
    while admitted <= certified_release_serving::ADMITTED_BYTES {
        let Some((paged, count)) = reached(world, swept, admitted, failures) else {
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
fn assert_limits_are_reported_as_limits(
    killed: impl Fn() -> PendingWalWorld,
    need: u64,
    stage: &str,
) {
    let mut failures = Vec::new();
    let entries = entry_need(&killed(), &mut failures);
    let manifest = byte_need(&killed(), Swept::ManifestBytes, &mut failures);
    let bytes = byte_need(&killed(), Swept::ObservationBytes, &mut failures);
    let staging = byte_need(&killed(), Swept::StagingBytes, &mut failures);
    assert!(
        failures.is_empty(),
        "{stage}: {} blocks under {entries:?} entries, {manifest:?} manifest bytes, \
         {bytes:?} observation bytes and {staging:?} staging bytes did not name the limit:\n{}",
        failures.len(),
        failures.join("\n"),
    );
    assert_eq!(entries, Some(need), "{stage}: the entry need is fixed");
    assert!(
        manifest.is_some_and(|bytes| bytes > RECORD_STRIDE),
        "{stage}: no manifest-byte limit blocked: {manifest:?}",
    );
    assert!(
        bytes.is_some_and(|bytes| bytes > PAGE_STRIDE),
        "{stage}: no observation-byte limit blocked: {bytes:?}",
    );
    assert!(
        staging.is_some_and(|bytes| bytes > PAGE_STRIDE),
        "{stage}: no staging-byte limit blocked: {staging:?}",
    );
}

#[test]
fn every_limit_under_the_need_of_published_objects_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(idle_world, IDLE_NEED, "published above the checkpoint");
}

#[test]
fn every_limit_under_the_need_of_a_historical_release_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        released_world,
        RELEASED_NEED,
        "released above the checkpoint",
    );
}

#[test]
fn every_limit_under_the_need_of_a_release_above_a_checkpoint_head_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        pending_successor_above_history::terminal_successor_of_a_checkpoint_head,
        HEADED_NEED,
        "released above a checkpoint head",
    );
}
