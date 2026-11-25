use super::{
    RuntimeWorldDurableMedium, WorldDurableArtifact, WorldDurableHomeBinding,
    WorldDurableMediumDenial, WorldPublicationObservation,
};
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurablePublicationFate;
use worth_foundational::facade::WorldDurableJournal;
use worth_proof::AuthorityMarker;
mod capture_reservation;
mod committed;
mod refused;
pub(crate) use refused::{ActualWorldNoEffectPublication, WorldPublicationRefusal};
mod home_program;
pub(crate) use committed::WorldCommittedPublication;
pub(crate) use home_program::HomeProgramBindingReservation;

/// The exact subject acknowledged by the attached medium. Its constructor is
/// private to this binding's actual Committed branch.
#[derive(Debug)]
pub(crate) struct ActualWorldPublication {
    subject: WorldDurableArtifact,
}
impl AuthorityMarker for ActualWorldPublication {}
impl ActualWorldPublication {
    pub(crate) fn into_subject(self) -> WorldDurableArtifact {
        self.subject
    }
}
#[derive(Debug)]
pub(crate) enum WorldDurablePublicationStop {
    Pending,
    Refused(WorldPublicationRefusal),
    Unresolved,
    EvidenceRequired,
    /// A prior subject already left this journal behind a recovery barrier, so
    /// the incoming subject was never submitted to the medium.
    BlockedByPriorBarrier {
        evidence_required: bool,
    },
    Busy,
}
#[derive(Debug, Default)]
struct JournalFlight {
    subject: Option<WorldDurableArtifact>,
    in_caller: bool,
    barrier: Option<PublicationBarrier>,
}
#[derive(Debug, Clone, Copy)]
enum PublicationBarrier {
    PhysicalIndeterminate,
    EvidenceRequired,
}
#[derive(Debug, Default)]
struct PublicationGates {
    branches: JournalFlight,
    occurrences: JournalFlight,
    custody: JournalFlight,
    capture_reserved: bool,
    home_binding_reserved: bool,
}
impl PublicationGates {
    fn journal(&mut self, journal: WorldDurableJournal) -> &mut JournalFlight {
        match journal {
            WorldDurableJournal::Branches => &mut self.branches,
            WorldDurableJournal::Occurrences => &mut self.occurrences,
            WorldDurableJournal::Custody => &mut self.custody,
        }
    }
}
pub(crate) type PreparedOccurrenceLocation =
    worth_execution::ExecutionOrderedMapCell<u64, (u64, [u8; 32])>;

/// One attached writer. Unresolved state has no reset; actual reopen constructs
/// a new binding only after physical recovery and Home admission.
#[derive(Debug)]
pub(crate) struct WorldDurableMediumBinding {
    medium: Arc<dyn RuntimeWorldDurableMedium>,
    current_home_binding: std::sync::OnceLock<WorldDurableHomeBinding>,
    gates: Mutex<PublicationGates>,
    pub(in crate::durability) checkpoint_acknowledgement:
        Mutex<super::checkpoint::WorldCheckpointAcknowledgement>,
    pub(in crate::durability) checkpoint_origin:
        Mutex<super::checkpoint::WorldCheckpointOriginState>,
    occurrence_locations: Mutex<worth_execution::ExecutionOrderedMap<u64, (u64, [u8; 32])>>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorldDurableMediumBinding {
    pub(crate) fn new(
        medium: Arc<dyn RuntimeWorldDurableMedium>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, WorldDurableMediumDenial> {
        let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed World binding layout");
        let funding = request
            .reserve_retained_memory(layout.pad_to_align().size() as u64)
            .map_err(WorldDurableMediumDenial::Request)?;
        Ok(Arc::new(Self {
            medium,
            current_home_binding: std::sync::OnceLock::new(),
            gates: Mutex::new(PublicationGates::default()),
            checkpoint_acknowledgement: Mutex::new(Default::default()),
            checkpoint_origin: Mutex::new(Default::default()),
            occurrence_locations: Mutex::new(worth_execution::ExecutionOrderedMap::default()),
            _funding: funding,
        }))
    }
    pub(crate) fn retire_medium_for_home(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldDurableMediumDenial> {
        self.medium.retire_for_home(request)
    }
    pub(crate) fn binding(&self) -> &WorldDurableHomeBinding {
        self.current_home_binding
            .get()
            .unwrap_or_else(|| self.medium.binding())
    }

    pub(crate) fn install_home_binding(&self, binding: WorldDurableHomeBinding) {
        self.current_home_binding
            .set(binding)
            .expect("the before-service Home receiver is promoted once");
    }
    pub(crate) fn prepare_occurrence_location(
        attempt_ordinal: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedOccurrenceLocation, WorldDurableMediumDenial> {
        worth_execution::ExecutionOrderedMapCell::prepare_for_request(
            attempt_ordinal,
            (0, [0; 32]),
            *request,
        )
        .map_err(WorldDurableMediumDenial::Ordered)
    }
    pub(crate) fn install_prepared_occurrence_location(
        &self,
        cell: PreparedOccurrenceLocation,
        position: u64,
        digest: [u8; 32],
    ) {
        let key = *cell.key();
        let rejected = {
            let mut locations = self
                .occurrence_locations
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            match locations.insert_vacant_prepared(cell) {
                Ok(()) => {
                    let _ = locations.set_existing_unique(&key, (position, digest));
                    None
                }
                Err((_, cell)) => Some(cell),
            }
        };
        drop(rejected);
    }
    pub(crate) fn install_recovered_occurrence_locations(
        &self,
        recovered: worth_execution::ExecutionOrderedMap<u64, (u64, [u8; 32])>,
    ) {
        let previous = {
            let mut locations = self
                .occurrence_locations
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            std::mem::replace(&mut *locations, recovered)
        };
        drop(previous);
    }
    pub(crate) fn occurrence_location(&self, attempt_ordinal: u64) -> Option<(u64, [u8; 32])> {
        self.occurrence_locations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&attempt_ordinal)
            .copied()
    }
    pub(crate) fn capture_checkpoint_source(
        self: &Arc<Self>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn super::checkpoint::WorldCheckpointReadSource>, WorldDurableMediumDenial>
    {
        let reservation = self.reserve_checkpoint_capture()?;
        let source = self.medium.capture_checkpoint_source(request);
        drop(reservation);
        source
    }
    pub(crate) fn admit_recovered_checkpoint_coverage(
        &self,
        coverage: super::checkpoint::WorldCheckpointCoverage,
    ) -> Result<(), WorldDurableMediumDenial> {
        self.checkpoint_acknowledgement
            .try_lock()
            .map_err(|_| WorldDurableMediumDenial::PublicationBusy)?
            .admit_recovered(coverage)
    }
    pub(crate) fn publication_recovery_required(&self) -> bool {
        let gates = self.gates.lock().unwrap_or_else(|e| e.into_inner());
        gates.custody.barrier.is_some()
            || gates.occurrences.barrier.is_some()
            || gates.branches.barrier.is_some()
    }
    pub(crate) fn occurrence_unresolved(&self) -> bool {
        self.gates
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .occurrences
            .barrier
            .is_some()
    }
    pub(crate) fn try_admit_close(&self) -> Result<(), crate::lifecycle::RuntimeWorldCloseDenial> {
        use crate::lifecycle::RuntimeWorldCloseDenial as Denial;
        let gates = self.gates.try_lock().map_err(|_| Denial::Busy)?;
        if gates.capture_reserved || gates.home_binding_reserved {
            return Err(Denial::Busy);
        }
        if gates.branches.in_caller || gates.custody.in_caller || gates.occurrences.in_caller {
            return Err(Denial::DurablePublicationUnsettled);
        }
        match self.medium.publication_task_posture() {
            super::WorldPublicationTaskPosture::Busy => Err(Denial::Busy),
            super::WorldPublicationTaskPosture::Unsettled => {
                Err(Denial::DurablePublicationUnsettled)
            }
            super::WorldPublicationTaskPosture::Terminal => Ok(()),
        }
    }
    pub(crate) fn publish(
        self: &Arc<Self>,
        subject: &WorldDurableArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<WorldCommittedPublication, WorldDurablePublicationStop> {
        {
            let mut gates = self.gates.lock().unwrap_or_else(|e| e.into_inner());
            if gates.capture_reserved || gates.home_binding_reserved {
                return Err(WorldDurablePublicationStop::Busy);
            }
            let flight = gates.journal(subject.journal());
            if let Some(barrier) = flight.barrier {
                let evidence_required = matches!(barrier, PublicationBarrier::EvidenceRequired);
                let same_subject = flight
                    .subject
                    .as_ref()
                    .is_some_and(|original| original.same_subject(subject));
                if !same_subject {
                    return Err(WorldDurablePublicationStop::BlockedByPriorBarrier {
                        evidence_required,
                    });
                }
                return Err(if evidence_required {
                    WorldDurablePublicationStop::EvidenceRequired
                } else {
                    WorldDurablePublicationStop::Unresolved
                });
            }
            if flight.in_caller {
                return Err(WorldDurablePublicationStop::Busy);
            }
            match &flight.subject {
                Some(original) if !original.same_subject(subject) => {
                    return Err(WorldDurablePublicationStop::Busy)
                }
                Some(_) => {}
                None => flight.subject = Some(subject.clone()),
            }
            flight.in_caller = true;
        }
        let mut guard = PublicationUnwind {
            binding: self.clone(),
            subject: subject.clone(),
            armed: true,
        };
        // No gate or product-reference lock is held over the adapter call.
        let observation = self.medium.advance_publication(subject, execution);
        let result = {
            let mut gates = self.gates.lock().unwrap_or_else(|e| e.into_inner());
            let flight = gates.journal(subject.journal());
            flight.in_caller = false;
            match observation {
                WorldPublicationObservation::Pending => Err(WorldDurablePublicationStop::Pending),
                WorldPublicationObservation::EvidenceRequired => {
                    flight.barrier = Some(PublicationBarrier::EvidenceRequired);
                    Err(WorldDurablePublicationStop::EvidenceRequired)
                }
                WorldPublicationObservation::Finished(completion) => {
                    let (fate, location) = completion.into_parts();
                    match fate {
                        DurablePublicationFate::Committed => {
                            let (journal, position, digest) = location
                                .expect("committed constructor retains its actual location");
                            let position = position.get();
                            if journal != subject.journal() {
                                flight.barrier = Some(PublicationBarrier::PhysicalIndeterminate);
                                Err(WorldDurablePublicationStop::Unresolved)
                            } else {
                                match committed::matches_digest(subject, digest, execution) {
                                    Ok(true) => {
                                        let original = flight
                                            .subject
                                            .take()
                                            .expect("registered original publication");
                                        Ok(WorldCommittedPublication::new(
                                            original, position, digest,
                                        ))
                                    }
                                    Ok(false) => {
                                        flight.barrier =
                                            Some(PublicationBarrier::PhysicalIndeterminate);
                                        Err(WorldDurablePublicationStop::Unresolved)
                                    }
                                    Err(_) => {
                                        flight.barrier = Some(PublicationBarrier::EvidenceRequired);
                                        Err(WorldDurablePublicationStop::EvidenceRequired)
                                    }
                                }
                            }
                        }
                        DurablePublicationFate::ProvenNoEffect { detail } => {
                            flight.subject = None;
                            Err(WorldDurablePublicationStop::Refused(
                                WorldPublicationRefusal::new(subject.clone(), detail),
                            ))
                        }
                        DurablePublicationFate::Indeterminate { detail: _ } => {
                            flight.barrier = Some(PublicationBarrier::PhysicalIndeterminate);
                            Err(WorldDurablePublicationStop::Unresolved)
                        }
                    }
                }
            }
        };
        guard.armed = false;
        result
    }
}
struct PublicationUnwind {
    binding: Arc<WorldDurableMediumBinding>,
    subject: WorldDurableArtifact,
    armed: bool,
}
impl Drop for PublicationUnwind {
    fn drop(&mut self) {
        if self.armed {
            let mut gates = self.binding.gates.lock().unwrap_or_else(|e| e.into_inner());
            let flight = gates.journal(self.subject.journal());
            flight.barrier = Some(PublicationBarrier::PhysicalIndeterminate);
            flight.in_caller = false;
        }
    }
}
