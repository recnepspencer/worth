//! A recovery admitted under less than its world needs is told which limit
//! ran out and the value it was admitted under, at every limit below the
//! need and whichever phase met it. None of them is told its media is
//! damaged. The same media answers every probe: a blocked recovery has no
//! effect.
//!
//! Manifest entries, manifest bytes, observation bytes and staging bytes
//! are each swept from one up to the need.

pub use {std::path::Path, crate::production_entry::certified_release_serving, crate::production_entry::pending_successor_above_history, crate::production_entry::pending_wal_world, worth_store_court_recovery_runtime_replication_production_entry_accumulator_only_pending_wal::production_entry::published_above_checkpoint, crate::production_entry::tier_release_pending_wal};
pub use {crate::production_entry::published_above_checkpoint::manifest_entry_limit::released_world, crate::production_entry::published_above_checkpoint::manifest_entry_limit::IDLE_NEED, crate::production_entry::published_above_checkpoint::manifest_entry_limit::ORDERED_NEED, crate::production_entry::published_above_checkpoint::manifest_entry_limit::RELEASED_NEED};
pub use {crate::production_entry::published_above_checkpoint::limit_sweep::entries::entry_need, crate::production_entry::published_above_checkpoint::limit_sweep::entries::History, crate::production_entry::published_above_checkpoint::limit_sweep::entries::Need, crate::production_entry::published_above_checkpoint::limit_sweep::entries::Staging};
pub use {crate::production_entry::pending_wal_world::PendingWalWorld};
pub use {crate::production_entry::published_above_checkpoint::limit_sweep::worlds::Killed};
pub use {worth_store_recovery_runtime::PhysicalRecoveryBlock, worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryLimitDimension, worth_store_recovery_runtime::PhysicalRecoveryOpenRequest, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};






/// What the first reopen needs where a checkpoint heads the released
/// object: the release gate then replays the head path its roster holds.
/// Every root read costs one entry, found or not. Among them: the successor
/// root the candidate probe finds absent (1), and the historical roots a
/// blob record is looked up under (4).
pub const HEADED_NEED: u64 = 95;

/// A certified release reopened with nothing pending: the gate replays its
/// checkpoint head and nothing after it.
pub const CERTIFIED_NEED: u64 = 39;

/// The released world once its successor was sealed and served: the walk
/// orders that completed history above the same checkpoint.
pub const COMPLETED_NEED: u64 = 232;

/// A tier release killed after its descriptor WAL: the gate reads the
/// released directory's records and redoes the pending release.
pub const TIER_NEED: u64 = 50;

/// One checkpointed record rewritten inline: completing it rereads the one
/// moved segment.
pub const REWRITE_NEED: u64 = 6;

/// One extent copied to another arena: completing it rereads the copy's
/// chunks.
pub const COPY_NEED: u64 = 11;

/// The entries the two-chunk worlds admit.
pub const SUFFICIENT_ENTRIES: u64 = 4096;

/// How far a byte sweep moves past a block that names the limit without
/// the count its read reached. A page admission rereads whole pages of
/// these worlds; discovery and the phases after the walk read records.
pub const PAGE_STRIDE: u64 = 8192;
pub const RECORD_STRIDE: u64 = 128;

#[derive(Clone, Copy, Debug)]
pub enum Swept {
    ManifestEntries,
    ManifestBytes,
    ObservationBytes,
    StagingBytes,
}

impl Swept {
    pub fn request(self, root: &Path, admitted: u64) -> PhysicalRecoveryOpenRequest {
        crate::production_entry::certified_release_serving::recovery_request::request_narrowing(root, |declared| match self {
            Self::ManifestEntries => declared.manifest_entries = admitted,
            Self::ManifestBytes => declared.manifest_bytes = admitted,
            Self::ObservationBytes => declared.observation_bytes = admitted,
            Self::StagingBytes => declared.staging_bytes = admitted,
        })
    }

    pub fn dimension(self) -> PhysicalRecoveryLimitDimension {
        match self {
            Self::ManifestEntries => worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ManifestEntries,
            Self::ManifestBytes => worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ManifestBytes,
            Self::ObservationBytes => worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ObservationBytes,
            Self::StagingBytes => worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::StagingBytes,
        }
    }
}

pub fn idle_world() -> PendingWalWorld {
    pending_wal_world::published_above_checkpoint(Workload::ThreeSmallObjects, Tail::Idle)
}

/// Whether this block says anything but that a limit ran out. Discovery,
/// which counts no planning, keeps the reader failure that met the limit as
/// the source of its block; planning names the limit in its denial, or
/// names nothing.
pub fn names_damage(blocked: &PhysicalRecoveryBlock) -> bool {
    let evidence = blocked.evidence();
    if blocked.cause().limit().is_some() && evidence.planning_counters.is_none() {
        return evidence.planning_denial.is_some();
    }
    !evidence.source_denials.is_empty()
        || evidence
            .planning_denial
            .as_ref()
            .is_some_and(|denial| !crate::published_above_checkpoint::limit_sweep::named::only_a_limit(denial))
}

/// How this block falls short of naming the swept limit, if it does.
pub fn shortfall(blocked: &PhysicalRecoveryBlock, swept: Swept, admitted: u64) -> Option<String> {
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
pub fn paged(blocked: &PhysicalRecoveryBlock) -> bool {
    blocked.cause().phase() == worth_store_recovery_runtime::PhysicalRecoveryBlockKind::PageAdmission
}

/// What a probe that did not recover was told.
pub struct Blocked {
    /// Whether page admission blocked.
    pub paged: bool,
    /// The count the block says its charge or read reached, or one past the
    /// limit where it names none.
    pub count: u64,
    /// The phase that blocked and the outermost denial it named.
    pub denial: String,
}

/// What the probe under `admitted` was told; `None` once the world
/// recovers. A block that does not name the limit is recorded against the
/// limit it was admitted under.
pub fn reached(
    root: &Path,
    swept: Swept,
    admitted: u64,
    failures: &mut Vec<String>,
) -> Option<Blocked> {
    match worth_store_recovery_runtime::WorthStoreRecovery::recover(swept.request(root, admitted)) {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => {
            drop(handoff);
            None
        }
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => {
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
pub fn byte_need(root: &Path, swept: Swept, failures: &mut Vec<String>) -> Option<u64> {
    let mut admitted = 1;
    while admitted <= crate::production_entry::certified_release_serving::recovery_request::ADMITTED_BYTES {
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
pub fn assert_limits_are_reported_as_limits<W: Killed>(
    killed: impl Fn() -> W,
    need: Need,
    stage: &str,
) {
    let mut failures = Vec::new();
    let entries = entry_need(&killed, need, &mut failures);
    let manifest = byte_need(killed().root(), crate::production_entry::published_above_checkpoint::limit_sweep::Swept::ManifestBytes, &mut failures);
    let bytes = byte_need(killed().root(), crate::production_entry::published_above_checkpoint::limit_sweep::Swept::ObservationBytes, &mut failures);
    let staging = byte_need(killed().root(), crate::production_entry::published_above_checkpoint::limit_sweep::Swept::StagingBytes, &mut failures);
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
        crate::production_entry::published_above_checkpoint::limit_sweep::entries::Staging::Staged => assert!(
            staging.is_some_and(|bytes| bytes > PAGE_STRIDE),
            "{stage}: no staging-byte limit blocked: {staging:?}",
        ),
        crate::production_entry::published_above_checkpoint::limit_sweep::entries::Staging::Unstaged => assert_eq!(staging, Some(1), "{stage}: nothing is staged"),
    }
}