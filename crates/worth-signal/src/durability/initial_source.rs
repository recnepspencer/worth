//! The original program and exclusive admission survive initial source publication.
mod driving;
use super::publication_binding::SignalCommittedAdvanceRecord;
use super::readmission::{
    SignalAdvanceLocationReservation, SignalDefinitionLocationReservation,
    SignalDefinitionSourceRecord,
};
use super::*;
use crate::logic::transaction::SignalRuntime;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{
    DurablePublicationFate, DurableSignalBranchIdentity,
    DurableSignalConditionalDefinitionOccurrence, SignalDefinitionTransitionIdentity,
    SignalDurableJournal,
};

/// Read-only actual program source and owner-issued coordinates, before any I/O.
pub struct SignalInitialDefinitionContext<'a> {
    pub(crate) graph: &'a crate::data::graph::SignalGraph,
    pub(crate) identity: SignalDefinitionTransitionIdentity,
    pub(crate) branch: DurableSignalBranchIdentity,
    pub(crate) occurrences: &'a [DurableSignalConditionalDefinitionOccurrence],
}
impl SignalInitialDefinitionContext<'_> {
    pub fn graph(&self) -> &crate::data::graph::SignalGraph {
        self.graph
    }
    pub fn identity(&self) -> SignalDefinitionTransitionIdentity {
        self.identity
    }
    pub fn branch(&self) -> DurableSignalBranchIdentity {
        self.branch
    }
    pub fn conditional_occurrences(&self) -> &[DurableSignalConditionalDefinitionOccurrence] {
        self.occurrences
    }
}

#[derive(Debug)]
pub enum SignalInitialSourceDenial {
    Readmission(SignalOwnerReadmissionDenial),
    Frame(SignalDurableFrameDenial),
    Image(SignalAuthoredImageDenial),
    Medium(SignalDurableMediumDenial),
    ProgramNotInitial,
    SourceNotEmpty { definitions: u64, advances: u64 },
    BridgeSectionRefused,
}
/// Preparation has performed no publication and returns the actual program.
pub struct SignalInitialSourcePreparationRefusal<Program> {
    pub(crate) program: Box<Program>,
    pub(crate) cause: SignalInitialSourceDenial,
}
impl<Program> SignalInitialSourcePreparationRefusal<Program> {
    pub fn cause(&self) -> &SignalInitialSourceDenial {
        &self.cause
    }
    pub fn into_parts(self) -> (Box<Program>, SignalInitialSourceDenial) {
        (self.program, self.cause)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalInitialSourceProgress {
    Pending,
    Published,
}
/// The fate belongs to this journal step. The whole initialization stays retained.
#[derive(Debug)]
pub struct SignalInitialSourceStepRefusal {
    pub journal: SignalDurableJournal,
    pub fate: DurablePublicationFate<crate::branch::owner_services::SignalDurableAdvanceDenial>,
}

pub(crate) enum InitialSourcePhase {
    Definition,
    Advance(Arc<SignalDefinitionSourceRecord>),
    Published(SignalCommittedAdvanceRecord),
}
pub(crate) struct InitialSourceCustody<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) program: Box<SignalRuntime<D, I, E, Ctx, T>>,
    pub(crate) medium: Arc<dyn SignalDurableMedium>,
    pub(crate) contribution: SignalRecordedIdentityContribution,
    pub(crate) vocabulary: SignalDurableTierVocabulary<T>,
    pub(crate) admission: SignalDurableOwnerAdmission,
    pub(crate) definition: SignalDefinitionTransitionArtifact,
    pub(crate) advance: SignalBranchAdvanceArtifact,
    pub(crate) definition_custody: Option<SignalDefinitionLocationReservation>,
    pub(crate) _advance_custody: SignalAdvanceLocationReservation,
    pub(crate) _memory: Option<ExecutionMemoryReservation>,
}
/// Store this original before driving it. Disposing a borrowed observation does
/// not dispose either publication, the program, or its exclusive admission.
#[must_use]
pub struct SignalInitialSourcePublication<D, I, E, Ctx, T = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) custody: Box<InitialSourceCustody<D, I, E, Ctx, T>>,
    pub(crate) phase: InitialSourcePhase,
}
/// Actual initial branch ACK plus the original program and exclusive admission.
/// Descriptive bytes cannot construct this serving handoff.
#[must_use]
pub struct SignalPublishedInitialSource<D, I, E, Ctx, T = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    custody: Box<InitialSourceCustody<D, I, E, Ctx, T>>,
    record: SignalCommittedAdvanceRecord,
}
