use super::{
    RuntimeWorldDurableMedium, WorldDurableArtifact, WorldDurableHomeBinding,
    WorldDurableMediumDenial, WorldPublicationObservation,
};
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{DurablePublicationFate, WorldDurableJournal};
use worth_proof::{AuthorityMarker, DurablyCommitted};

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
    Refused(WorldDurableMediumDenial),
    Unresolved,
    Busy,
}
#[derive(Debug, Default)]
struct JournalFlight {
    subject: Option<WorldDurableArtifact>,
    in_caller: bool,
    unresolved: bool,
}
#[derive(Debug, Default)]
struct PublicationGates {
    branches: JournalFlight,
    occurrences: JournalFlight,
    custody: JournalFlight,
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
/// One attached writer. Unresolved state has no reset; actual reopen constructs
/// a new binding only after physical recovery and Home admission.
#[derive(Debug)]
pub(crate) struct WorldDurableMediumBinding {
    medium: Box<dyn RuntimeWorldDurableMedium>,
    gates: Mutex<PublicationGates>,
}
impl WorldDurableMediumBinding {
    pub(crate) fn new(medium: Box<dyn RuntimeWorldDurableMedium>) -> Arc<Self> {
        Arc::new(Self {
            medium,
            gates: Mutex::new(PublicationGates::default()),
        })
    }
    pub(crate) fn binding(&self) -> &WorldDurableHomeBinding {
        self.medium.binding()
    }
    pub(crate) fn recover(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<super::WorldDurableRecoverySource, WorldDurableMediumDenial> {
        self.medium.recover(execution)
    }
    pub(crate) fn occurrence_unresolved(&self) -> bool {
        self.gates
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .occurrences
            .unresolved
    }
    pub(crate) fn publish(
        self: &Arc<Self>,
        subject: &WorldDurableArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<DurablyCommitted<ActualWorldPublication>, WorldDurablePublicationStop> {
        {
            let mut gates = self.gates.lock().unwrap_or_else(|e| e.into_inner());
            let flight = gates.journal(subject.journal());
            if flight.unresolved {
                return Err(WorldDurablePublicationStop::Unresolved);
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
                WorldPublicationObservation::Finished(DurablePublicationFate::Committed) => {
                    let original = flight
                        .subject
                        .take()
                        .expect("registered original publication");
                    Ok(DurablyCommitted::from_owner(ActualWorldPublication {
                        subject: original,
                    }))
                }
                WorldPublicationObservation::Finished(DurablePublicationFate::ProvenNoEffect {
                    detail,
                }) => {
                    flight.subject = None;
                    Err(WorldDurablePublicationStop::Refused(detail))
                }
                WorldPublicationObservation::Finished(DurablePublicationFate::Indeterminate {
                    detail: _,
                }) => {
                    flight.unresolved = true;
                    Err(WorldDurablePublicationStop::Unresolved)
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
            flight.unresolved = true;
            flight.in_caller = false;
        }
    }
}
