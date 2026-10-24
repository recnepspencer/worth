//! The canonical commit journal publishes at the next position under one
//! durable root decision, answers point and range reads, resolves positions
//! from durable state, and survives a killed process on both sides of the
//! first checkpoint.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, CanonicalCommitPublication,
    CanonicalCommitPublicationFailure, CanonicalCommitPublished, CanonicalCommitResolution,
    ManifestEntryCapacity, PhysicalRecordFormatDeclaration, PhysicalRecordPlacementPolicy,
    ServingPhysicalRuntime,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

use super::*;

const ROLE_ENV: &str = "WORTH_JOURNAL_CHILD_ROLE";
const MARKER_ENV: &str = "WORTH_JOURNAL_CHILD_MARKER";
const CHILD_TEST: &str = "canonical_journal::canonical_journal_child";
const ENTRIES: u64 = 12;
const CHECKPOINT_AFTER: u64 = 7;

fn payload(position: u64) -> Vec<u8> {
    let length = 64 + (position as usize * 1_237) % 9_000;
    (0..length)
        .map(|index| (position as u8) ^ (index as u8).rotate_left(3))
        .collect()
}

/// The placement the recovery world initializes with.
fn placement() -> AdmittedRecordPlacementPolicy {
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ))
        .unwrap()
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn publication(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    position: u64,
    bytes: Vec<u8>,
) -> Result<CanonicalCommitPublished, CanonicalCommitPublicationFailure> {
    serving
        .canonical_commits()
        .expect("journal access")
        .publish(CanonicalCommitPublication::new(
            position,
            bytes,
            placement,
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
}

/// Requires positions 1..=head to read back exactly, newest first in range.
fn assert_journal_holds(serving: &ServingPhysicalRuntime, head: u64) {
    let journal = serving.canonical_commits().expect("journal access");
    let observed = journal
        .head()
        .expect("head read")
        .map(|entry| entry.position());
    assert_eq!(observed, (head > 0).then_some(head));
    for position in 1..=head {
        let read = journal.read(position, 1 << 20).expect("point read");
        assert_eq!(read.entry().position(), position);
        assert!(
            read.bytes() == payload(position),
            "payload {position} changed"
        );
        assert!(matches!(
            journal.resolve(position, digest(&payload(position))).unwrap(),
            CanonicalCommitResolution::Committed(entry) if entry.position() == position
        ));
        assert!(matches!(
            journal.resolve(position, [0xee; 32]).unwrap(),
            CanonicalCommitResolution::Conflict(_)
        ));
    }
    assert_eq!(
        journal
            .resolve(head + 1, digest(&payload(head + 1)))
            .unwrap(),
        CanonicalCommitResolution::NotCommitted { head }
    );
    let range = journal
        .range_descending(head, usize::MAX)
        .expect("range read");
    let positions: Vec<_> = range.iter().map(|entry| entry.position()).collect();
    assert_eq!(positions, (1..=head).rev().collect::<Vec<_>>());
}

#[test]
fn the_journal_publishes_only_at_the_next_position_and_resolves_retries() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("journal-in-process")
        .expect("journal store");
    let serving = world.serving();
    assert_journal_holds(serving, 0);
    assert!(matches!(
        publication(serving, world.placement(), 2, payload(2)),
        Err(CanonicalCommitPublicationFailure::PositionGap {
            head: 0,
            requested: 2
        })
    ));
    for position in 1..=ENTRIES {
        let published = publication(serving, world.placement(), position, payload(position))
            .expect("publication");
        assert!(matches!(published, CanonicalCommitPublished::Committed(_)));
        assert_eq!(published.entry().position(), position);
    }
    assert_journal_holds(serving, ENTRIES);
    assert!(matches!(
        publication(serving, world.placement(), 5, payload(5)),
        Ok(CanonicalCommitPublished::AlreadyCommitted(entry)) if entry.position() == 5
    ));
    assert!(matches!(
        publication(serving, world.placement(), 5, payload(6)),
        Err(CanonicalCommitPublicationFailure::PositionConflict(entry)) if entry.position() == 5
    ));
    assert!(matches!(
        publication(
            serving,
            world.placement(),
            ENTRIES + 2,
            payload(ENTRIES + 2)
        ),
        Err(CanonicalCommitPublicationFailure::PositionGap { .. })
    ));
    assert_journal_holds(serving, ENTRIES);
}

/// Where the killed child stopped. Paused stops park the last publication at
/// a mutation seam, so recovery itself must settle that entry.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stop {
    BeforeFirstCheckpoint,
    AboveCheckpoint,
    BeforeWalAppend,
    AfterWalDurability,
    AfterDataSettlement,
    DuringRootPublication,
    AfterRootReplacement,
}

impl Stop {
    const ALL: [Self; 7] = [
        Self::BeforeFirstCheckpoint,
        Self::AboveCheckpoint,
        Self::BeforeWalAppend,
        Self::AfterWalDurability,
        Self::AfterDataSettlement,
        Self::DuringRootPublication,
        Self::AfterRootReplacement,
    ];

    const fn role(self) -> &'static str {
        match self {
            Self::BeforeFirstCheckpoint => "before-first-checkpoint",
            Self::AboveCheckpoint => "above-checkpoint",
            Self::BeforeWalAppend => "before-wal-append",
            Self::AfterWalDurability => "after-wal-durability",
            Self::AfterDataSettlement => "after-data-settlement",
            Self::DuringRootPublication => "during-root-publication",
            Self::AfterRootReplacement => "after-root-replacement",
        }
    }

    fn seam(self) -> Option<CertificationPhysicalMutationCheckpoint> {
        use CertificationPhysicalMutationCheckpoint as Seam;
        match self {
            Self::BeforeFirstCheckpoint | Self::AboveCheckpoint => None,
            Self::BeforeWalAppend => Some(Seam::BeforeWalAppend),
            Self::AfterWalDurability => Some(Seam::AfterWalDurability),
            Self::AfterDataSettlement => Some(Seam::AfterDataSettlement),
            Self::DuringRootPublication => Some(Seam::DuringRootPublication),
            Self::AfterRootReplacement => Some(Seam::AfterRootReplacement),
        }
    }

    /// The head recovery must select: a publication with a durable WAL record
    /// rolls forward; one stopped before its WAL append left no effect.
    const fn recovered_head(self) -> u64 {
        match self {
            Self::BeforeWalAppend => ENTRIES - 1,
            _ => ENTRIES,
        }
    }
}

fn launch(stop: Stop) -> (PathBuf, tempfile::TempDir) {
    let marker_dir = tempfile::tempdir().unwrap();
    let marker = marker_dir.path().join("ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, stop.role())
        .env(MARKER_ENV, &marker)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(240);
    while !marker.is_file() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "journal child did not park: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = PathBuf::from(fs::read_to_string(&marker).unwrap());
    assert_eq!(
        root.join("families/checkpoint.current").exists(),
        stop == Stop::AboveCheckpoint
    );
    (root, marker_dir)
}

fn serve(root: &Path, stage: &str) -> ServingPhysicalRuntime {
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("{stage} did not recover: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("journal custody seal");
    certified_release_serving::admit_serving_with_seal(root, seal)
}

/// Recovers, checks the selected head, publishes the next entry, and requires
/// a second recovery to reach the same journal.
fn assert_killed_journal_recovers(stop: Stop) {
    let (root, _marker) = launch(stop);
    let head = stop.recovered_head();
    let serving = serve(&root, stop.role());
    assert_journal_holds(&serving, head);
    let next = publication(&serving, placement(), head + 1, payload(head + 1))
        .expect("publication after recovery");
    assert_eq!(next.entry().position(), head + 1);
    serving.close();
    let serving = serve(&root, "second recovery");
    assert_journal_holds(&serving, head + 1);
    serving.close();
}

#[test]
fn a_journal_killed_before_its_first_checkpoint_recovers_every_entry() {
    assert_killed_journal_recovers(Stop::BeforeFirstCheckpoint);
}

#[test]
fn a_journal_killed_above_a_checkpoint_recovers_every_entry() {
    assert_killed_journal_recovers(Stop::AboveCheckpoint);
}

#[test]
fn a_publication_killed_at_each_mutation_seam_recovers_to_one_complete_head() {
    for stop in [
        Stop::BeforeWalAppend,
        Stop::AfterWalDurability,
        Stop::AfterDataSettlement,
        Stop::DuringRootPublication,
        Stop::AfterRootReplacement,
    ] {
        assert_killed_journal_recovers(stop);
    }
}

fn checkpoint(world: &PhysicalResidencyStoreWorld) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xc1; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("the checkpoint must start");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

fn child(marker: &Path, stop: Stop) -> ! {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("journal-killed")
        .expect("journal store");
    let settled = if stop.seam().is_some() {
        ENTRIES - 1
    } else {
        ENTRIES
    };
    for position in 1..=settled {
        publication(
            world.serving(),
            world.placement(),
            position,
            payload(position),
        )
        .expect("publication before the kill");
        if stop == Stop::AboveCheckpoint && position == CHECKPOINT_AFTER {
            checkpoint(&world);
        }
    }
    let announce = || {
        let pending = marker.with_extension("pending");
        fs::write(&pending, world.root().to_string_lossy().as_bytes()).unwrap();
        fs::rename(pending, marker).unwrap();
    };
    if let Some(seam) = stop.seam() {
        thread::scope(|scope| {
            let gate = world
                .serving()
                .certification_pause_physical_mutation_at(seam);
            scope.spawn(|| {
                let _ = publication(
                    world.serving(),
                    world.placement(),
                    ENTRIES,
                    payload(ENTRIES),
                );
            });
            assert!(
                gate.await_arrival(),
                "publication did not reach {}",
                stop.role()
            );
            announce();
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        })
    }
    announce();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

#[test]
fn canonical_journal_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let marker = PathBuf::from(std::env::var_os(MARKER_ENV).expect("child marker"));
    let stop = Stop::ALL
        .into_iter()
        .find(|stop| Some(stop.role()) == role.to_str())
        .expect("journal child role");
    child(&marker, stop);
}
