//! Original recorded transition meaning; only the Home admits its publication.
use worth_execution::{ExecutionArray, ExecutionMemoryReservation};
use worth_foundational::facade::*;
use worth_relational::facade::branch::RecordedRelationalComponentReference;

mod decoding;
mod encoding;
mod fields;
pub(super) use decoding::decode;
pub(super) use encoding::encode;
pub(super) const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
pub(super) const MAGIC: &[u8; 8] = b"WHMIGINT";
pub(super) const MAXIMUM: usize = 64 << 10;

/// The complete immutable original intent. These coordinates are descriptive;
/// they cannot mint permission to execute a step or declare one durable.
#[derive(Debug)]
pub struct HomeProgramTransitionIntent {
    pub(super) plan: HomeProgramTransitionPlanDescription,
    pub(super) predecessor: ApplicationHomeBinding,
    pub(super) successor: ApplicationHomeBinding,
    pub(super) native: RecordedRelationalComponentReference,
    pub(super) bodies: std::sync::Arc<super::native_bodies::HomeNativeStepBodies>,
    pub(super) assignments: ExecutionArray<CompositeAttemptAssociation>,
    pub(super) _metadata: Option<ExecutionMemoryReservation>,
}
impl DurableArtifactRetention for HomeProgramTransitionIntent {
    const RETENTION_CLASS: RetentionClass = RetentionClass::Authoritative;
}
impl HomeProgramTransitionIntent {
    pub(crate) fn program_steps(
        &self,
    ) -> impl ExactSizeIterator<Item = HomeProgramTransitionProgramStep> + '_ {
        self.assignments()
            .iter()
            .enumerate()
            .map(|(index, original)| {
                HomeProgramTransitionProgramStep::from_recorded(
                    original.clone(),
                    self.predecessor().selected().revision(),
                    if index <= self.plan().target_meaning().step() as usize {
                        self.predecessor().selected().revision()
                    } else {
                        self.successor().selected().revision()
                    },
                )
            })
    }
    pub fn plan(&self) -> &HomeProgramTransitionPlanDescription {
        &self.plan
    }
    pub fn predecessor(&self) -> &ApplicationHomeBinding {
        &self.predecessor
    }
    pub fn successor(&self) -> &ApplicationHomeBinding {
        &self.successor
    }
    pub fn native_predecessor(&self) -> &RecordedRelationalComponentReference {
        &self.native
    }
    pub fn bodies(&self) -> &[HomeProgramTransitionBodyDigest] {
        self.bodies.values()
    }
    pub fn assignments(&self) -> &[CompositeAttemptAssociation] {
        self.assignments.elements()
    }
}

#[derive(Debug)]
pub enum ApplicationHomeTransitionIntentDenial {
    Execution(worth_execution::WorkCeilingDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    Binding(crate::ApplicationHomeDescriptorDenial),
    Native(worth_relational::facade::branch::RelationalRecordedComponentRefusal),
    UnsupportedVersion { observed: BoundaryProtocolVersion },
    SizeLimit,
    Malformed,
    ForeignHome,
    ForeignIntentPosition,
    ForeignPredecessor,
    AssignmentCount,
    AssignmentOrder,
    TransitionPending,
    TransitionPlanDrift,
    Journal(worth_store::physical_runtime::OrderedJournalDenial),
    Read(worth_store::physical_runtime::OrderedJournalReadFailure),
    Deadline(crate::ApplicationHomeDeadlineDenial),
    Interruption(worth_execution::ExecutionInterruptionViewDenial),
    Busy,
    SourceMismatch,
    ReadSource(crate::ApplicationHomeCheckpointDenial),
    BindingLineage,
    ReservationOverlap,
    ReservationExhausted,
    TransitionNotRequested,
    IntentNotCommitted,
    StaleIncarnation,
    WorldAssociation(worth_runtime_world::facade::WorldDurableMediumDenial),
    StepAlreadyClaimed,
    TransitionResumeRequired,
    ForeignWorld,
    WorldLineage(worth_runtime_world::facade::WorldReadmissionDenial),
    JournalUnresolved,
    JournalSourceChanged,
}
impl HomeProgramTransitionIntent {
    pub(super) fn require_plan(
        &self,
        successor: &ApplicationHomeBinding,
        digest: HomeProgramTransitionPlanDigest,
        steps: u32,
        target: HomeProgramTransitionTargetMeaning,
        bodies: &[HomeProgramTransitionBodyDigest],
    ) -> Result<(), ApplicationHomeTransitionIntentDenial> {
        use ApplicationHomeTransitionIntentDenial as Denial;
        // The pending successor takes precedence over an edited author.
        if &self.successor != successor {
            return Err(Denial::TransitionPending);
        }
        if self.plan.digest() != digest
            || self.plan.steps() != steps
            || self.plan.target_meaning() != target
            || self.bodies() != bodies
        {
            return Err(Denial::TransitionPlanDrift);
        }
        Ok(())
    }
}
