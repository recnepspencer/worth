//! One original prepared subject; identifiers and copied bytes cannot mint it.
use super::WorldCheckpointCoverage;
use crate::durability::{WorldDurableHomeBinding, WorldDurableMediumDenial as Denial};
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionRequest};

#[derive(Clone)]
pub struct WorldCheckpointSubject {
    meaning: Arc<WorldCheckpointSubjectMeaning>,
}
struct WorldCheckpointSubjectMeaning {
    bytes: ExecutionImmutableBytes,
    binding: WorldDurableHomeBinding,
    coverage: WorldCheckpointCoverage,
    _funding: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for WorldCheckpointSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldCheckpointSubject")
            .field("coverage", &self.meaning.coverage)
            .field("bytes", &self.meaning.bytes.len())
            .finish_non_exhaustive()
    }
}
impl WorldCheckpointSubject {
    pub(super) fn new(
        request: &ExecutionRequest<'_, '_>,
        bytes: ExecutionImmutableBytes,
        binding: WorldDurableHomeBinding,
        coverage: WorldCheckpointCoverage,
    ) -> Result<Self, Denial> {
        let quote = std::mem::size_of::<WorldCheckpointSubjectMeaning>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .ok_or(Denial::Unavailable)?;
        let funding = request
            .reserve_retained_memory(quote as u64)
            .map_err(Denial::Request)?;
        Ok(Self {
            meaning: Arc::new(WorldCheckpointSubjectMeaning {
                bytes,
                binding,
                coverage,
                _funding: funding,
            }),
        })
    }
    pub fn same_capture(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.meaning, &other.meaning)
    }
    pub(super) fn bytes(&self) -> &[u8] {
        self.meaning.bytes.as_ref()
    }
    pub(super) fn coverage(&self) -> &WorldCheckpointCoverage {
        &self.meaning.coverage
    }
    pub(super) fn binding(&self) -> &WorldDurableHomeBinding {
        &self.meaning.binding
    }
}

/// Issued only by the closed Store constructor path after actual Home receipt,
/// selected full-manifest digest, section and SAME subject validation.
pub struct WorldCheckpointSelection {
    subject: WorldCheckpointSubject,
    generation: u64,
    manifest_sha256: [u8; 32],
}
impl WorldCheckpointSelection {
    /// Raw values do not establish this claim. The mechanically closed caller
    /// must consume its actual selected Home evidence before invoking this.
    pub fn from_home_checkpoint_selection(
        subject: &WorldCheckpointSubject,
        generation: u64,
        manifest_sha256: [u8; 32],
    ) -> Self {
        Self {
            subject: subject.clone(),
            generation,
            manifest_sha256,
        }
    }
    pub(super) fn admits(&self, subject: &WorldCheckpointSubject) -> bool {
        self.generation != 0 && self.subject.same_capture(subject)
    }
    pub(super) fn generation(&self) -> u64 {
        self.generation
    }
    pub(super) fn manifest_sha256(&self) -> [u8; 32] {
        self.manifest_sha256
    }
}
