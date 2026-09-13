use std::collections::{BTreeMap, BTreeSet};
use worth_store_physical_format::{
    ExtentChunkCoordinate, PersistedPhysicalDataFrameSubject, PersistedRecordIdentity,
    PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalSegmentId, RecordFrameCoordinate,
};

use super::{PhysicalRedoPlanningDenial, PhysicalRedoTarget, PhysicalRedoTargetIdentity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryPageObservation {
    target: PhysicalRedoTargetIdentity,
    page_lsn: u64,
    frame_digest: [u8; 32],
    absent_prior: bool,
    source: RecoveryPageSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPageSource {
    Materialized {
        coordinate: RecordFrameCoordinate,
        routing_identity: [u8; 32],
    },
    AbsentTarget {
        coordinate: RecordFrameCoordinate,
        root_membership_identity: [u8; 32],
    },
    PlannedResult {
        coordinate: RecordFrameCoordinate,
        causal_identity: [u8; 32],
    },
}

impl RecoveryPageObservation {
    pub const fn materialized(
        target: PhysicalRedoTargetIdentity,
        page_lsn: u64,
        frame_digest: [u8; 32],
        coordinate: RecordFrameCoordinate,
        routing_identity: [u8; 32],
    ) -> Self {
        Self {
            target,
            page_lsn,
            frame_digest,
            absent_prior: false,
            source: RecoveryPageSource::Materialized {
                coordinate,
                routing_identity,
            },
        }
    }
    pub fn absent(target: &PhysicalRedoTarget, root_membership_identity: [u8; 32]) -> Self {
        let (_, coordinate) = target_format_basis(target);
        Self {
            target: target.identity(),
            page_lsn: 0,
            frame_digest: absent_digest(target),
            absent_prior: true,
            source: RecoveryPageSource::AbsentTarget {
                coordinate,
                root_membership_identity,
            },
        }
    }
    pub const fn target(&self) -> PhysicalRedoTargetIdentity {
        self.target
    }
    pub const fn page_lsn(&self) -> u64 {
        self.page_lsn
    }
    pub const fn frame_digest(&self) -> [u8; 32] {
        self.frame_digest
    }
    pub const fn is_absent_prior(&self) -> bool {
        self.absent_prior
    }
    pub const fn source(&self) -> RecoveryPageSource {
        self.source
    }
}

pub(super) struct RecoveryPageCursor {
    pages: BTreeMap<PhysicalRedoTargetLocation, CursorPage>,
    observed_predecessors: BTreeSet<(u64, PhysicalRedoTargetIdentity)>,
}

struct CursorPage {
    observation: RecoveryPageObservation,
    last_apply_group: Option<[u8; 32]>,
    last_claim: Option<PhysicalRedoTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PhysicalRedoTargetLocation {
    InlinePage { segment: u64, page: u64 },
    ExtentChunk { extent: u64, chunk: u32 },
}

impl RecoveryPageCursor {
    pub(super) fn new(
        observations: Vec<RecoveryPageObservation>,
    ) -> Result<Self, PhysicalRedoPlanningDenial> {
        let mut pages = BTreeMap::new();
        for observation in observations {
            if pages
                .insert(
                    location(observation.target()),
                    CursorPage {
                        observation,
                        last_apply_group: None,
                        last_claim: None,
                    },
                )
                .is_some()
            {
                return Err(PhysicalRedoPlanningDenial::InvalidTarget);
            }
        }
        Ok(Self {
            pages,
            observed_predecessors: BTreeSet::new(),
        })
    }

    pub(super) fn retain_observed_predecessors(
        &mut self,
        predecessors: BTreeSet<(u64, PhysicalRedoTargetIdentity)>,
    ) {
        self.observed_predecessors = predecessors;
    }

    pub(super) fn observe_record(
        &self,
        target: PhysicalRedoTargetIdentity,
        lsn: u64,
    ) -> Result<RecoveryPageObservation, PhysicalRedoPlanningDenial> {
        if self.observed_predecessors.contains(&(lsn, target)) {
            return self
                .pages
                .get(&location(target))
                .map(|page| page.observation)
                .ok_or(PhysicalRedoPlanningDenial::MissingPageObservation);
        }
        self.observe(target)
    }

    pub(super) fn observe(
        &self,
        target: PhysicalRedoTargetIdentity,
    ) -> Result<RecoveryPageObservation, PhysicalRedoPlanningDenial> {
        let page = self
            .pages
            .get(&location(target))
            .ok_or(PhysicalRedoPlanningDenial::MissingPageObservation)?;
        let observed_generation = generation(page.observation.target());
        let target_generation = generation(target);
        if (!page.observation.is_absent_prior() || page.observation.target() != target)
            && observed_generation != target_generation
            && observed_generation.checked_add(1) != Some(target_generation)
        {
            return Err(PhysicalRedoPlanningDenial::GenerationMismatch);
        }
        Ok(page.observation)
    }

    pub(super) fn advance(
        &mut self,
        operation: [u8; 32],
        target: &PhysicalRedoTarget,
        page_lsn: u64,
    ) -> Result<(), PhysicalRedoPlanningDenial> {
        let page = self
            .pages
            .get_mut(&location(target.identity()))
            .ok_or(PhysicalRedoPlanningDenial::MissingPageObservation)?;
        let current_generation = generation(page.observation.target());
        let target_generation = generation(target.identity());
        let exact_successor = current_generation.checked_add(1) == Some(target_generation);
        let exact_absent_target =
            page.observation.is_absent_prior() && page.observation.target() == target.identity();
        let same_group_final_claim = current_generation == target_generation
            && page.last_apply_group == Some(operation)
            && page.last_claim.as_ref() == Some(target);
        if !exact_absent_target && !exact_successor && !same_group_final_claim {
            return Err(PhysicalRedoPlanningDenial::GenerationMismatch);
        }
        let (_, coordinate) = target_format_basis(target);
        page.observation = RecoveryPageObservation {
            target: target.identity(),
            page_lsn,
            frame_digest: target.resulting_digest(),
            absent_prior: false,
            source: RecoveryPageSource::PlannedResult {
                coordinate,
                causal_identity: planned_result_identity(
                    page.observation,
                    operation,
                    target,
                    page_lsn,
                ),
            },
        };
        page.last_apply_group = Some(operation);
        page.last_claim = Some(target.clone());
        Ok(())
    }
}

fn absent_digest(target: &PhysicalRedoTarget) -> [u8; 32] {
    let (subject, coordinate) = target_format_basis(target);
    worth_store_physical_format::certified_absent_prior_image_digest(subject, coordinate)
}

fn target_format_basis(
    target: &PhysicalRedoTarget,
) -> (PersistedPhysicalDataFrameSubject, RecordFrameCoordinate) {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let subject = match target.identity() {
        PhysicalRedoTargetIdentity::InlinePage {
            segment,
            page,
            generation,
        } => {
            let segment_id = PhysicalSegmentId::from_raw(segment)
                .expect("verified redo target carries nonzero segment");
            let page_id =
                PhysicalPageId::from_raw(page).expect("verified redo target carries nonzero page");
            let generation = PhysicalGeneration::from_raw(generation)
                .expect("verified redo target carries nonzero generation");
            PersistedPhysicalDataFrameSubject::InlinePage(
                authority
                    .page_cell(segment_id, page_id)
                    .with_page_generation(generation),
            )
        }
        PhysicalRedoTargetIdentity::ExtentChunk {
            extent,
            generation,
            chunk,
        } => {
            let coordinate = target
                .extent_coordinate()
                .expect("verified extent redo target carries exact coordinate");
            let record = PersistedRecordIdentity::new(
                coordinate.allocation_epoch(),
                coordinate.record_ordinal(),
            )
            .expect("verified extent redo target carries record identity");
            let extent_id = PhysicalExtentId::from_raw(extent)
                .expect("verified redo target carries nonzero extent");
            let generation = PhysicalGeneration::from_raw(generation)
                .expect("verified redo target carries nonzero generation");
            let extent_cell = authority
                .record_extent_cell(extent_id)
                .with_extent_generation(generation);
            PersistedPhysicalDataFrameSubject::ExtentChunk(
                ExtentChunkCoordinate::new(
                    record,
                    extent_cell,
                    coordinate.logical_bytes(),
                    coordinate.logical_offset(),
                    chunk,
                )
                .expect("verified extent redo target carries canonical chunk coordinate"),
            )
        }
    };
    let coordinate = RecordFrameCoordinate::new(
        target.artifact(),
        target.artifact_offset(),
        target.artifact_length(),
    )
    .expect("verified redo target carries canonical frame coordinate");
    (subject, coordinate)
}

fn planned_result_identity(
    prior: RecoveryPageObservation,
    operation: [u8; 32],
    target: &PhysicalRedoTarget,
    page_lsn: u64,
) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"worth.store.recovery.planned-page-transition.v1");
    digest.update(prior.frame_digest());
    digest.update(operation);
    digest.update(page_lsn.to_le_bytes());
    digest.update(target.resulting_digest());
    digest.finalize().into()
}

const fn location(identity: PhysicalRedoTargetIdentity) -> PhysicalRedoTargetLocation {
    match identity {
        PhysicalRedoTargetIdentity::InlinePage { segment, page, .. } => {
            PhysicalRedoTargetLocation::InlinePage { segment, page }
        }
        PhysicalRedoTargetIdentity::ExtentChunk { extent, chunk, .. } => {
            PhysicalRedoTargetLocation::ExtentChunk { extent, chunk }
        }
    }
}

const fn generation(identity: PhysicalRedoTargetIdentity) -> u64 {
    match identity {
        PhysicalRedoTargetIdentity::InlinePage { generation, .. }
        | PhysicalRedoTargetIdentity::ExtentChunk { generation, .. } => generation,
    }
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
