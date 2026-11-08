//! Per-branch original tasks share only short metadata locks and physical publication order.
mod composite;
mod driving;
mod frame_preparation;
pub(in crate::branch::owner_services) mod medium_binding;
mod port;
mod preparation;
pub(in crate::branch::owner_services) mod prepared;
mod source_access;
mod task;
mod task_storage;
mod transaction_result;
use super::{SignalOwner, SignalOwnerUnavailable};
use crate::branch::retention::SignalBranchAdmissionReservation;
use crate::durability::readmission::{
    SignalAdvanceLocationReservation, SignalDefinitionLocationReservation,
};
use crate::durability::readmission::{
    SignalDefinitionSourceRecord, SignalRecoveredSourceLocations,
};
use crate::durability::{
    SignalBranchAdvanceArtifact, SignalDefinitionTransitionArtifact, SignalDurableMedium,
    SignalDurableTierVocabulary,
};
use crate::state::SignalBranchId;
pub use port::SignalDurableBranchMutationPort;
use prepared::SignalPreparedDurableAdvance;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak},
};
pub use task::{
    SignalDurableAdvanceCompletion, SignalDurableAdvanceDenial, SignalDurableAdvanceDriveRefusal,
    SignalDurableAdvanceOutput, SignalDurableAdvanceProgress, SignalDurableAdvanceTask,
};
pub use transaction_result::SignalDurableTransactionResult;
use worth_foundational::facade::DurablePublicationFate;
use worth_foundational::facade::{DurableSignalAdmissionIdentity, SignalBranchAdvanceIdentity};
use worth_proof::TerminalState;
use worth_proof::{AuthorityMarker, AuthorityWitness, LinearResource};

struct SignalTaskTermination {
    fate: DurablePublicationFate<()>,
}
impl TerminalState for SignalTaskTermination {
    fn label(&self) -> &'static str {
        match self.fate {
            DurablePublicationFate::Committed => "committed",
            DurablePublicationFate::ProvenNoEffect { detail: () } => "proven-no-effect",
            DurablePublicationFate::Indeterminate { detail: () } => "indeterminate",
        }
    }
}

// This is live owner custody, not a serializable authority marker or a new
// substrate witness. The proof substrate supplies the linear resource law.
struct SignalPublicationTaskOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    _owner: Weak<SignalOwner<D, I, T>>,
}
impl<D, I, T> AuthorityMarker for SignalPublicationTaskOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
}
struct ReadyAdvance<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    prepared: SignalPreparedDurableAdvance<D, I, T>,
    artifact: SignalBranchAdvanceArtifact,
    retention: SignalBranchAdmissionReservation,
    location: SignalAdvanceLocationReservation,
    publication: Option<Box<dyn crate::durability::SignalDurableDefinitionPublication>>,
}
struct DefinitionPending<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    advance: ReadyAdvance<D, I, T>,
    definition: SignalDefinitionTransitionArtifact,
    location: SignalDefinitionLocationReservation,
}
struct BranchPending<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    advance: ReadyAdvance<D, I, T>,
    definition: Arc<SignalDefinitionSourceRecord>,
}
enum TaskPhase<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    Preparing,
    Definition(Box<DefinitionPending<D, I, T>>),
    Branch(Box<BranchPending<D, I, T>>),
    Completing,
    Completion(Box<SignalDurableAdvanceCompletion>),
    Delivered,
}
struct TaskSlot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    branch: SignalBranchId,
    resource: Option<
        LinearResource<
            SignalBranchAdvanceIdentity,
            SignalTaskTermination,
            SignalPublicationTaskOwner<D, I, T>,
        >,
    >,
    phase: TaskPhase<D, I, T>,
    _composite_grant: Option<worth_proof::AdmittedCompositeSignalPublication>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
struct TaskTable<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    branches: BTreeMap<SignalBranchId, SignalBranchAdvanceIdentity>,
    tasks: BTreeMap<SignalBranchAdvanceIdentity, Arc<Mutex<TaskSlot<D, I, T>>>>,
}
pub(crate) struct SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    owner: Weak<SignalOwner<D, I, T>>,
    medium: Arc<dyn SignalDurableMedium>,
    vocabulary: Arc<SignalDurableTierVocabulary<T>>,
    source: Mutex<SignalRecoveredSourceLocations>,
    tasks: Mutex<TaskTable<D, I, T>>,
    physical_turn: Mutex<Option<SignalBranchAdvanceIdentity>>,
    unsettled: Mutex<Option<SignalBranchAdvanceIdentity>>,
    authority: AuthorityWitness<SignalPublicationTaskOwner<D, I, T>>,
    composite: Mutex<Option<worth_proof::InstalledCompositeSignalPublicationAuthority>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(crate) fn new(
        owner: Weak<SignalOwner<D, I, T>>,
        medium: Arc<dyn SignalDurableMedium>,
        vocabulary: Arc<SignalDurableTierVocabulary<T>>,
        source: SignalRecoveredSourceLocations,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, crate::durability::SignalOwnerReadmissionDenial> {
        let bytes = crate::data::retained_storage::arc_allocation_charge::<Self>()
            .map_err(|_| crate::durability::SignalOwnerReadmissionDenial::MetadataCapacity)?
            .bytes();
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(crate::durability::SignalOwnerReadmissionDenial::Request)?;
        let authority = AuthorityWitness::from_authority_marker(SignalPublicationTaskOwner {
            _owner: owner.clone(),
        });
        Ok(Arc::new(Self {
            owner,
            medium,
            vocabulary,
            source: Mutex::new(source),
            tasks: Mutex::new(TaskTable {
                branches: BTreeMap::new(),
                tasks: BTreeMap::new(),
            }),
            physical_turn: Mutex::new(None),
            unsettled: Mutex::new(None),
            authority,
            composite: Mutex::new(None),
            _memory: memory,
        }))
    }
    fn owner(&self) -> Result<Arc<SignalOwner<D, I, T>>, SignalDurableAdvanceDenial> {
        SignalOwner::upgrade(&self.owner)
            .map_err(|SignalOwnerUnavailable| SignalDurableAdvanceDenial::OwnerUnavailable)
    }
    fn task(
        &self,
        original: SignalBranchAdvanceIdentity,
    ) -> Result<Arc<Mutex<TaskSlot<D, I, T>>>, SignalDurableAdvanceDenial> {
        if original.owner() != self.medium.binding().owner() {
            return Err(SignalDurableAdvanceDenial::ForeignTask);
        }
        self.tasks
            .lock()
            .map_err(|_| SignalDurableAdvanceDenial::PublicationStatePoisoned)?
            .tasks
            .get(&original)
            .cloned()
            .ok_or(SignalDurableAdvanceDenial::UnknownTask { original })
    }
    fn finish_resource(
        &self,
        original: SignalBranchAdvanceIdentity,
        mut slot: TaskSlot<D, I, T>,
        completion: SignalDurableAdvanceCompletion,
    ) -> SignalDurableAdvanceCompletion {
        let resource = slot
            .resource
            .take()
            .expect("one original task reaches one terminal answer");
        slot.phase = TaskPhase::Delivered;
        if let DurablePublicationFate::Indeterminate { .. } = completion.fate() {
            *self
                .unsettled
                .lock()
                .expect("unsettled original contains only an identity") = Some(original);
        }
        let mut tasks = self
            .tasks
            .lock()
            .expect("terminal task metadata contains only owner primitives");
        tasks.branches.remove(&slot.branch);
        tasks.tasks.remove(&original);
        drop(tasks);
        let mut turn = self
            .physical_turn
            .lock()
            .expect("physical turn contains only an original identity");
        if *turn == Some(original) {
            *turn = None;
        }
        drop(turn);
        let fate = match completion.fate() {
            DurablePublicationFate::Committed => DurablePublicationFate::Committed,
            DurablePublicationFate::ProvenNoEffect { .. } => {
                DurablePublicationFate::ProvenNoEffect { detail: () }
            }
            DurablePublicationFate::Indeterminate { .. } => {
                DurablePublicationFate::Indeterminate { detail: () }
            }
        };
        let (identity, _) = resource
            .terminate(SignalTaskTermination { fate })
            .into_parts();
        debug_assert_eq!(identity, completion.original());
        completion
    }
}
impl<D, I, T> Drop for TaskSlot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            let fate = match self.phase {
                TaskPhase::Completion(_) => DurablePublicationFate::Committed,
                TaskPhase::Preparing => DurablePublicationFate::ProvenNoEffect { detail: () },
                TaskPhase::Definition(_)
                | TaskPhase::Branch(_)
                | TaskPhase::Completing
                | TaskPhase::Delivered => DurablePublicationFate::Indeterminate { detail: () },
            };
            let _terminal = resource.terminate(SignalTaskTermination { fate });
        }
    }
}
