//! The canonical commit journal publishes at the next position under one
//! durable root decision, answers point and range reads, resolves positions
//! from durable state, and survives a killed process on both sides of the
//! first checkpoint.

pub use {
    std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio,
    std::thread, std::time::Duration, std::time::Instant,
};

pub use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
pub use {sha2::Digest, sha2::Sha256};
pub use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::OrderedJournalPublication,
    worth_store::physical_runtime::OrderedJournalPublicationFailure,
    worth_store::physical_runtime::OrderedJournalPublished,
    worth_store::physical_runtime::OrderedJournalResolution,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub use {
    crate::production_entry::certified_release_serving, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

pub const ROLE_ENV: &str = "WORTH_JOURNAL_CHILD_ROLE";
pub const MARKER_ENV: &str = "WORTH_JOURNAL_CHILD_MARKER";
pub const CHILD_TEST: &str = "process::recovery_runtime::production_entry::ordered_journals::canonical_journal::canonical_journal_child";

pub const ENTRIES: u64 = 12;
pub const CHECKPOINT_AFTER: u64 = 7;

pub fn payload(position: u64) -> Vec<u8> {
    let length = 64 + (position as usize * 1_237) % 9_000;
    (0..length)
        .map(|index| (position as u8) ^ (index as u8).rotate_left(3))
        .collect()
}

/// The placement the recovery world initializes with.
pub fn placement() -> AdmittedRecordPlacementPolicy {
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(
            worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
                worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
                    .admit()
                    .unwrap(),
            ),
        )
        .unwrap()
}

pub fn digest(bytes: &[u8]) -> [u8; 32] {
    sha2::Sha256::digest(bytes).into()
}

pub fn publication(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    position: u64,
    bytes: Vec<u8>,
) -> Result<OrderedJournalPublished, OrderedJournalPublicationFailure> {
    serving
        .canonical_commits()
        .expect("journal access")
        .publish(
            worth_store::physical_runtime::OrderedJournalPublication::new(
                position,
                bytes,
                placement,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
}

/// Requires positions 1..=head to read back exactly, newest first in range.
pub fn assert_journal_holds(serving: &ServingPhysicalRuntime, head: u64) {
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
            worth_store::physical_runtime::OrderedJournalResolution::Committed(entry) if entry.position() == position
        ));
        assert!(matches!(
            journal.resolve(position, [0xee; 32]).unwrap(),
            worth_store::physical_runtime::OrderedJournalResolution::Conflict(_)
        ));
    }
    assert_eq!(
        journal
            .resolve(head + 1, digest(&payload(head + 1)))
            .unwrap(),
        worth_store::physical_runtime::OrderedJournalResolution::NotCommitted { head }
    );
    let range = journal
        .range_descending(head, usize::MAX)
        .expect("range read");
    let positions: Vec<_> = range.iter().map(|entry| entry.position()).collect();
    assert_eq!(positions, (1..=head).rev().collect::<Vec<_>>());
}

/// Where the killed child stopped. Paused stops park the last publication at
/// a mutation seam, so recovery itself must settle that entry.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    BeforeFirstCheckpoint,
    AboveCheckpoint,
    BeforeWalAppend,
    AfterWalDurability,
    AfterDataSettlement,
    DuringRootPublication,
    AfterRootReplacement,
}

impl Stop {
    pub const ALL: [Self; 7] = [
        Self::BeforeFirstCheckpoint,
        Self::AboveCheckpoint,
        Self::BeforeWalAppend,
        Self::AfterWalDurability,
        Self::AfterDataSettlement,
        Self::DuringRootPublication,
        Self::AfterRootReplacement,
    ];

    pub const fn role(self) -> &'static str {
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

    pub fn seam(self) -> Option<CertificationPhysicalMutationCheckpoint> {
        use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
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
    pub const fn recovered_head(self) -> u64 {
        match self {
            Self::BeforeWalAppend => ENTRIES - 1,
            _ => ENTRIES,
        }
    }
}

pub fn launch(stop: Stop) -> (PathBuf, tempfile::TempDir) {
    let marker_dir = tempfile::tempdir().unwrap();
    let marker = marker_dir.path().join("ready");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, stop.role())
        .env(MARKER_ENV, &marker)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
    while !marker.is_file() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "journal child did not park: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = std::path::PathBuf::from(std::fs::read_to_string(&marker).unwrap());
    assert_eq!(
        root.join("families/checkpoint.current").exists(),
        stop == crate::production_entry::ordered_journals::canonical_journal::Stop::AboveCheckpoint
    );
    (root, marker_dir)
}

pub fn serve(root: &Path, stage: &str) -> ServingPhysicalRuntime {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        crate::production_entry::certified_release_serving::recovery_request::request(root),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("{stage} did not recover: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("journal custody seal");
    crate::production_entry::certified_release_serving::admit_serving_with_seal(root, seal)
}

/// Recovers, checks the selected head, publishes the next entry, and requires
/// a second recovery to reach the same journal.
pub fn assert_killed_journal_recovers(stop: Stop) {
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

pub fn checkpoint(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xc1; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("the checkpoint must start");
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}

#[expect(
    clippy::disallowed_methods,
    reason = "Runs independent journal publication actors at the controlled crash boundary."
)]
pub fn child(marker: &Path, stop: Stop) -> ! {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("journal-killed")
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
        if stop
            == crate::production_entry::ordered_journals::canonical_journal::Stop::AboveCheckpoint
            && position == CHECKPOINT_AFTER
        {
            checkpoint(&world);
        }
    }
    let announce = || {
        let pending = marker.with_extension("pending");
        std::fs::write(&pending, world.root().to_string_lossy().as_bytes()).unwrap();
        std::fs::rename(pending, marker).unwrap();
    };
    if let Some(seam) = stop.seam() {
        std::thread::scope(|scope| {
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
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        })
    }
    announce();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
