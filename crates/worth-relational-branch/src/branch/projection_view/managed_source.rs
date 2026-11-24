//! Exact selected Source and cache custody, independent of publication orchestration.
pub mod affinity;
pub mod correspondence;
pub mod history;
pub mod ordered_predecessor;
use crate::branch::projection_view::companion_slot::{CompanionBranchCellCore, CompanionRootImage};
use crate::history::data::coordinates::CommitId;
use crate::identity::data::{EntityId, VersionId};
use crate::validation::candidate_source::availability::ManagedIndexSourceAvailability;
use crate::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;
use crate::validation::candidate_source::image::{
    ManagedIndexBranchImage, ManagedIndexPublishedSourceRoot, ManagedIndexSourceAssociation,
    ManagedIndexSourceCacheRetirement,
};
use crate::validation::candidate_source::program::ManagedIndexSourceProgram;
use crate::visibility::materialization::read_records::reader::positioned_snapshot::PositionedRelationalSnapshot;
use std::{
    any::Any,
    sync::{Arc, Weak},
};

pub struct ManagedIndexSourceObservation {
    core: Weak<CompanionBranchCellCore>,
    affinity: Arc<affinity::ManagedSourceCellAffinity>,
    record: Arc<ManagedIndexSourceAssociation>,
    payload: Arc<dyn Any + Send + Sync>,
    selected: PositionedRelationalSnapshot,
    _metadata_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// One matching physical image follows the Native root's actual retained life.
/// The weak cell link cannot keep a newer branch cache alive through an old root.
#[derive(Clone)]
pub struct ManagedSourceRootImage {
    core: Weak<CompanionBranchCellCore>,
    affinity: Arc<affinity::ManagedSourceCellAffinity>,
    image: Arc<CompanionRootImage>,
    _fork_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}
impl std::fmt::Debug for ManagedSourceRootImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedSourceRootImage")
            .field("root", &self.image.root_id())
            .finish_non_exhaustive()
    }
}
impl ManagedSourceRootImage {
    pub(super) fn retain_image(
        core: &Arc<CompanionBranchCellCore>,
        image: &Arc<CompanionRootImage>,
    ) -> Option<Self> {
        Some(Self {
            core: Arc::downgrade(core),
            affinity: Arc::clone(core.source_affinity()?),
            image: Arc::clone(image),
            _fork_memory: core.fork_funding().cloned(),
        })
    }

    pub fn capture_current(core: &Arc<CompanionBranchCellCore>) -> Option<Self> {
        let current = core.read_image();
        Self::retain_image(core, current.image())
    }

    pub fn serving_cell(&self) -> Option<Arc<CompanionBranchCellCore>> {
        self.core.upgrade()
    }

    pub fn retained_image(&self) -> &CompanionRootImage {
        &self.image
    }

    pub(super) fn retained_affinity(&self) -> &Arc<affinity::ManagedSourceCellAffinity> {
        &self.affinity
    }

    pub(super) fn weak_cell(&self) -> Weak<CompanionBranchCellCore> {
        self.core.clone()
    }

    pub(crate) fn affinity(&self) -> &Arc<affinity::ManagedSourceCellAffinity> {
        self.retained_affinity()
    }

    pub fn selected_source_image(&self) -> Result<&ManagedIndexBranchImage, Denial> {
        self.image
            .payload::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)
    }
    pub fn history_owner(&self) -> Option<Arc<affinity::ManagedSourceCellAffinity>> {
        self.selected_source_image()
            .ok()
            .map(|_| Arc::clone(&self.affinity))
    }

    /// Cleanup uses the genuine selected image and current registration; it
    /// cannot manufacture an association or borrow Source content.
    pub fn evict_selected(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: &PositionedRelationalSnapshot,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        registration_generation: u64,
    ) -> Result<Option<ManagedIndexSourceCacheRetirement>, Denial> {
        self.admit_selected(selected, registration_generation)?;
        let image = self.selected_source_image()?;
        let retired = image
            .evict_source(execution, program, scope)
            .map_err(Denial::Publication)?;
        Ok(retired)
    }

    pub fn observe(
        &self,
        selected: PositionedRelationalSnapshot,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        registration_generation: u64,
    ) -> Result<ManagedIndexSourceObservation, Denial> {
        self.admit_selected(&selected, registration_generation)?;
        let image = self.selected_source_image()?;
        let record = image
            .retained(program, scope)?
            .ok_or(Denial::RootUnavailable)?;
        if record.physical_availability() != ManagedIndexSourceAvailability::Available {
            return Err(Denial::RootUnavailable);
        }
        let payload = record.payload().cloned().ok_or(Denial::RootUnavailable)?;
        Ok(self.observation(Arc::clone(&record.association), payload, selected, memory))
    }

    pub fn refill_selected(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: PositionedRelationalSnapshot,
        generation: u64,
        expected: &Arc<ManagedIndexPublishedSourceRoot>,
        payload: &Arc<dyn Any + Send + Sync>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<
        (
            ManagedIndexSourceObservation,
            ManagedIndexSourceCacheRetirement,
        ),
        Denial,
    > {
        self.admit_selected(&selected, generation)?;
        let image = self.selected_source_image()?;
        let retirement = image
            .refill_source(execution, expected, payload)
            .map_err(Denial::Publication)?;
        Ok((
            self.observation(
                Arc::clone(&expected.association),
                Arc::clone(payload),
                selected,
                memory,
            ),
            retirement,
        ))
    }

    fn observation(
        &self,
        record: Arc<ManagedIndexSourceAssociation>,
        payload: Arc<dyn Any + Send + Sync>,
        selected: PositionedRelationalSnapshot,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> ManagedIndexSourceObservation {
        ManagedIndexSourceObservation {
            core: self.core.clone(),
            affinity: Arc::clone(&self.affinity),
            record,
            payload,
            selected,
            _metadata_memory: memory,
        }
    }
}
impl ManagedIndexSourceObservation {
    pub(crate) fn selected_position(&self) -> &PositionedRelationalSnapshot {
        &self.selected
    }

    pub fn payload(&self) -> &Arc<dyn Any + Send + Sync> {
        &self.payload
    }
    /// Metadata affinity only. The receiving Query session still admits READ.
    pub fn matches_snapshot(&self, selected: &PositionedRelationalSnapshot) -> bool {
        &self.selected == selected
    }
    /// The Native publication that last replaced this complete Source support.
    /// Diagnostic identity alone cannot attach backing to a physical view.
    pub fn source_revision(&self) -> CommitId {
        self.record.revision
    }
    pub fn same_source(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.affinity, &other.affinity) && Arc::ptr_eq(&self.record, &other.record)
    }

    /// Correspondence of genuine retained Source observations, without claiming
    /// equal support, currentness, READ or permission to pin a different image.
    /// Forked cells may share payloads but never this actual affinity occurrence.
    pub fn same_source_scope(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.affinity, &other.affinity)
            && self.record.program == other.record.program
            && self.record.scope == other.record.scope
    }
}

/// Operations used by the Relational owners.
pub trait ManagedIndexSourceObservationOwnerPort: Sized {
    fn serving_cell(&self) -> Option<Arc<CompanionBranchCellCore>>;
    fn association(&self) -> &Arc<ManagedIndexSourceAssociation>;
    fn matches_publication_at(
        &self,
        runtime_identity: u64,
        commit: CommitId,
        version: VersionId,
    ) -> bool;
    /// Compare the complete retained Source support at an actual Native image.
    /// This is currentness only; it does not admit a principal or Source READ.
    fn remains_current_at(
        &self,
        selected: &PositionedRelationalSnapshot,
        registration_generation: u64,
    ) -> bool;
}

impl ManagedIndexSourceObservationOwnerPort for ManagedIndexSourceObservation {
    fn serving_cell(&self) -> Option<Arc<CompanionBranchCellCore>> {
        self.core.upgrade()
    }

    fn association(&self) -> &Arc<ManagedIndexSourceAssociation> {
        &self.record
    }

    fn matches_publication_at(
        &self,
        runtime_identity: u64,
        commit: CommitId,
        version: VersionId,
    ) -> bool {
        self.selected.runtime_instance_id() == runtime_identity
            && self.selected.commit_id() == Some(commit)
            && self.selected.version_id() == version
    }

    fn remains_current_at(
        &self,
        selected: &PositionedRelationalSnapshot,
        registration_generation: u64,
    ) -> bool {
        if affinity::admit_affinity(&self.affinity, selected, registration_generation).is_err() {
            return false;
        }
        let Some(core) = self.core.upgrade() else {
            return false;
        };
        let current = core.read_image();
        let current = current.image();
        current.root_id() == selected.root_id()
            && current.commit_id() == selected.commit_id()
            && current.position() == selected.position()
            && current
                .payload::<ManagedIndexBranchImage>()
                .is_some_and(|image| {
                    image
                        .retained(&self.record.program, self.record.scope)
                        .ok()
                        .flatten()
                        .is_some_and(|record| {
                            Arc::ptr_eq(&record.association, &self.record)
                                && record.physical_availability()
                                    == ManagedIndexSourceAvailability::Available
                        })
                })
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait ManagedIndexSourceObservationFixturePort: Sized {
    fn selected_position_for_test(&self) -> &PositionedRelationalSnapshot;
}

impl ManagedIndexSourceObservationFixturePort for ManagedIndexSourceObservation {
    fn selected_position_for_test(&self) -> &PositionedRelationalSnapshot {
        self.selected_position()
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait ManagedSourceRootImageFixturePort: Sized {
    fn affinity_for_test(&self) -> &Arc<affinity::ManagedSourceCellAffinity>;
}

impl ManagedSourceRootImageFixturePort for ManagedSourceRootImage {
    fn affinity_for_test(&self) -> &Arc<affinity::ManagedSourceCellAffinity> {
        self.affinity()
    }
}
