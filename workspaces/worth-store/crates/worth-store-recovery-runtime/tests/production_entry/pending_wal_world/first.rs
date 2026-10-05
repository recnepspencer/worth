//! First real killed producer, optionally using a bounded WAL segment size.

use super::*;
use worth_store::physical_runtime::{BlobReclaimDisposition, BlobTerminalLimits};

#[path = "first/launch.rs"]
mod launch;
#[path = "first/producer.rs"]
mod producer;
pub(crate) use launch::{
    first, first_with_failed_ingest_control, first_with_resume_frontier,
    first_with_wal_segment_bytes, published_above_checkpoint,
};
pub(super) use producer::child;

/// What the first child leaves selected when it is killed. Every object is
/// published above the baseline checkpoint and no later checkpoint covers it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum World {
    /// One two-chunk object under a parked release.
    TwoChunks,
    /// The two-chunk object, after a failed ingest left its drop controls.
    FailedIngestControl,
    /// One object with two resume frontiers, under a parked release: the
    /// ingest checkpoints one at its first chunk and another by itself 64
    /// chunks later, one chunk before its end.
    ResumeFrontier,
    /// Objects whose index maintenance filled and retired whole inline
    /// pages, killed idle or under a parked release of the first object.
    Published(Workload, Tail),
}

/// The objects a `World::Published` child publishes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Workload {
    TwentyFourChunks,
    ThreeSmallObjects,
    SixtySixChunks,
}

/// What the child is doing when it is killed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tail {
    /// Nothing: every publication completed.
    Idle,
    /// A release of the first object, parked at its durable descriptor WAL.
    PendingRelease,
}

impl World {
    const ALL: [Self; 9] = [
        Self::TwoChunks,
        Self::FailedIngestControl,
        Self::ResumeFrontier,
        Self::Published(Workload::TwentyFourChunks, Tail::Idle),
        Self::Published(Workload::TwentyFourChunks, Tail::PendingRelease),
        Self::Published(Workload::ThreeSmallObjects, Tail::Idle),
        Self::Published(Workload::ThreeSmallObjects, Tail::PendingRelease),
        Self::Published(Workload::SixtySixChunks, Tail::Idle),
        Self::Published(Workload::SixtySixChunks, Tail::PendingRelease),
    ];

    /// The child role that builds this world.
    pub(super) const fn role(self) -> &'static str {
        match self {
            Self::TwoChunks => "first",
            Self::FailedIngestControl => "mixed-first",
            Self::ResumeFrontier => "frontier-first",
            Self::Published(Workload::TwentyFourChunks, Tail::Idle) => "published-24",
            Self::Published(Workload::TwentyFourChunks, Tail::PendingRelease) => {
                "published-24-release"
            }
            Self::Published(Workload::ThreeSmallObjects, Tail::Idle) => "published-3x2",
            Self::Published(Workload::ThreeSmallObjects, Tail::PendingRelease) => {
                "published-3x2-release"
            }
            Self::Published(Workload::SixtySixChunks, Tail::Idle) => "published-66",
            Self::Published(Workload::SixtySixChunks, Tail::PendingRelease) => {
                "published-66-release"
            }
        }
    }

    pub(super) fn of_role(role: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|world| world.role() == role)
    }

    /// The first world of the root a later child was started on.
    pub(super) fn of_child_root() -> Self {
        std::env::var(FIRST_WORLD_ENV)
            .ok()
            .and_then(|role| Self::of_role(&role))
            .expect("first world of the child root")
    }

    /// The chunk count of each object, in publication order.
    const fn objects(self) -> &'static [usize] {
        match self {
            Self::TwoChunks | Self::FailedIngestControl => &[2],
            Self::Published(Workload::TwentyFourChunks, _) => &[24],
            Self::Published(Workload::ThreeSmallObjects, _) => &[2, 2, 2],
            Self::ResumeFrontier | Self::Published(Workload::SixtySixChunks, _) => &[66],
        }
    }

    /// Whether the killed child had parked a release of the first object.
    pub(super) const fn releases_first_object(self) -> bool {
        !matches!(self, Self::Published(_, Tail::Idle))
    }

    /// The bytes each object was published with, in publication order.
    pub(super) fn object_bytes(self) -> Vec<Vec<u8>> {
        self.objects()
            .iter()
            .enumerate()
            .map(|(object, chunks)| {
                (0..*chunks)
                    .flat_map(|ordinal| chunk(object, ordinal))
                    .collect()
            })
            .collect()
    }

    pub(super) fn recovery_request(
        self,
        root: &Path,
    ) -> worth_store_recovery_runtime::PhysicalRecoveryOpenRequest {
        match self {
            Self::TwoChunks
            | Self::FailedIngestControl
            | Self::Published(Workload::TwentyFourChunks | Workload::ThreeSmallObjects, _) => {
                super::super::certified_release_serving::request(root)
            }
            Self::ResumeFrontier | Self::Published(Workload::SixtySixChunks, _) => {
                super::super::certified_release_serving::request_for_long_ingest(root)
            }
        }
    }
}

/// Distinct whole chunks, so every one is its own selected record. The first
/// two of the first object are the bytes of the two-chunk world.
fn chunk(object: usize, ordinal: usize) -> Vec<u8> {
    let mut chunk = vec![if ordinal % 2 == 0 { 0x83 } else { 0x94 }; CHUNK];
    let salt = ((object * 128 + ordinal / 2) as u64).to_le_bytes();
    for (byte, salt) in chunk.iter_mut().zip(salt) {
        *byte ^= salt;
    }
    chunk
}

/// Where the child leaves the identities of the objects it published.
pub(super) fn objects_path(marker: &Path) -> PathBuf {
    marker.with_extension("objects")
}
