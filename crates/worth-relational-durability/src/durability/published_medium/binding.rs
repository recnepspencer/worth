use crate::durability::canonical_commit_artifact::journal_source_read::readmit_journal_commit;
use contracts::durability::medium::unresolved::{
    RelationalDurabilityUnresolved, RelationalDurabilityUnresolvedOwnerPort as _,
    RelationalDurabilityUnresolvedSubject,
};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port as storage;
pub mod source_retirement;
pub use source_retirement::SealedRelationalMediumBinding;
pub mod checkpoint_selection;
pub mod home_transition_requirement;
pub mod initial_inputs;
pub(crate) use checkpoint_selection::CheckpointPublication;
pub use home_transition_requirement::RelationalDurableMediumTransfer;
use home_transition_requirement::RelationalHomeProgramPhase;
pub use initial_inputs::RelationalCommittedInitialInputs;
pub use initial_inputs::RelationalInitialInputsPublicationStop;
use worth_proof::{AuthorityMarker, DurablyCommitted};

use super::DurablePublicationFate;
use super::RelationalDurableMedium;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use contracts::durability::medium::publication_refusal::RelationalDurablePublicationRefusal;
use storage::durability::medium::step_stop::RelationalDurableStepStop;
use worth_relational_branch::internal_port::branch::transition::durable_publication::RelationalDurableTransitionPublication;
use worth_relational_branch::internal_port::branch::transition::positioned::PositionedBranchTransition;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;

/// A runtime's attachment to its durable medium.
///
/// It owns the medium itself. The medium was moved into exactly one runtime
/// and leaves it only by moving into the runtime recovery rebuilds (D1); from
/// then on this attachment, and every port of the runtime that held it,
/// refuses to publish. It also owns the one fact the runtime keeps about the
/// medium between publications: whether an earlier publication's fate is
/// unknown. There is one such slot for commits and branch transitions alike:
/// once either is unresolved, every later publication of either kind refuses
/// with the same answer, so a later commit never overtakes an unresolved
/// delete and a later transition never overtakes an unresolved commit.
#[derive(Debug)]
pub struct RelationalDurableMediumBinding {
    transition: Option<worth_proof::HomeProgramTransitionAuthority>,
    world_selection: Option<
        Arc<
            worth_proof::VerifiedHomeWorldSelection<
                Arc<crate::durability::home_world_catalog::RelationalWorldSourceCatalog>,
            >,
        >,
    >,
    // Admission observes the writer phase without waiting for medium I/O.
    home_phase: Mutex<Option<RelationalHomeProgramPhase>>,
    state: Mutex<BindingState>,
    cold_source:
        Option<Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>>,
    unresolved: Arc<OnceLock<RelationalDurabilityUnresolved>>,
}

#[derive(Debug)]
struct BindingState {
    /// `None` once the medium moved to a recovered runtime.
    medium: Option<Box<dyn RelationalDurableMedium>>,
    unresolved: Arc<OnceLock<RelationalDurabilityUnresolved>>,
}

/// Placement facts issued only after the journal answered Committed.
#[derive(Debug)]
pub struct RelationalDurableCommitReadmission {
    positioned: Arc<PositionedCanonicalCommit>,
    entry: worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex,
    source: Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>,
}
impl RelationalDurableCommitReadmission {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Arc<PositionedCanonicalCommit>,
        worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex,
        Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>,
    ){
        (self.positioned, self.entry, self.source)
    }
}

impl AuthorityMarker for RelationalDurableCommitReadmission {}

impl RelationalDurableMediumBinding {
    pub fn has_unresolved_publication(&self) -> bool {
        self.unresolved.get().is_some()
    }
    pub fn cold_source(
        &self,
    ) -> Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource> {
        self.cold_source
            .as_ref()
            .expect("only the live binding exposes its source")
            .clone()
    }
    pub(super) fn preparation_fate(&self) -> Arc<OnceLock<RelationalDurabilityUnresolved>> {
        self.unresolved.clone()
    }
    pub(crate) fn read_transition(
        &self,
        sequence: contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    ) -> Result<
        RelationalBranchTransitionArtifact,
        storage::durability::medium::transition_read::RelationalDurableTransitionReadFailure,
    > {
        self.state()
            .medium
            .as_ref()
            .ok_or(storage::durability::medium::transition_read::RelationalDurableTransitionReadFailure::MediumMoved)?
            .read_transition(sequence)
    }
    pub fn new(transfer: RelationalDurableMediumTransfer) -> Self {
        let RelationalDurableMediumTransfer { medium, phase } = transfer;
        let unresolved = Arc::new(OnceLock::new());
        let cold_source = medium.cold_source();
        let transition = medium.home_transition_authority().cloned();
        let world_selection = medium.home_world_selection();
        Self {
            transition,
            world_selection,
            cold_source: Some(cold_source),
            home_phase: Mutex::new(Some(phase)),
            state: Mutex::new(BindingState {
                medium: Some(medium),
                unresolved: unresolved.clone(),
            }),
            unresolved,
        }
    }

    pub fn home_world_selection(
        &self,
    ) -> Option<
        Arc<
            worth_proof::VerifiedHomeWorldSelection<
                Arc<crate::durability::home_world_catalog::RelationalWorldSourceCatalog>,
            >,
        >,
    > {
        self.world_selection.clone()
    }

    pub fn home_transition_authority(
        &self,
    ) -> Option<&worth_proof::HomeProgramTransitionAuthority> {
        self.transition.as_ref()
    }

    /// Encode `commit` as its canonical artifact and make it durable.
    ///
    /// Called with the patch-position reservation held, after preparation and
    /// before the live cutover, so a refusal leaves no live or durable effect.
    pub fn publish(
        &self,
        commit: Arc<PositionedCanonicalCommit>,
        roots: &contracts::branch::target::RelationalBranchRootDescriptor,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DurablyCommitted<RelationalDurableCommitReadmission>, RelationalDurableStepStop>
    {
        let mut state = self.state();
        let medium = state.admitting_medium()?;
        let artifact =
            RelationalCanonicalCommitArtifact::from_positioned_request(&commit, execution)
                .map_err(|denial| refused(RelationalDurablePublicationRefusal::Encoding(denial)))?;
        let entry = worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex::from_publication(
            &commit,
            roots,
            *artifact.payload_digest().as_bytes(),
            execution,
            readmit_journal_commit,
        )
        .map_err(|denial| refused(RelationalDurablePublicationRefusal::Encoding(denial)))?;
        contracts::execution::request_custody::check_request(execution).map_err(|cause| {
            refused(RelationalDurablePublicationRefusal::Encoding(
                contracts::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial::Request(cause),
            ))
        })?;
        let answer = medium.publish(&artifact);
        state.settle(
            answer,
            RelationalDurabilityUnresolvedSubject::CanonicalCommit {
                position: artifact.position(),
                commit_id: artifact.commit_id(),
                digest: artifact.payload_digest(),
            },
            || RelationalDurableCommitReadmission {
                positioned: commit,
                entry,
                source: self.cold_source(),
            },
        )
    }

    /// Encode `transition` as its artifact and make it durable.
    ///
    /// Called with the patch-position reservation held, after every fallible
    /// preparation step and before any live effect of the transition.
    pub(crate) fn publish_transition(
        &self,
        transition: PositionedBranchTransition,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<DurablyCommitted<RelationalDurableTransitionPublication>, RelationalDurableStepStop>
    {
        self.admit_home_program_write(None, None).map_err(|cause| {
            refused(RelationalDurablePublicationRefusal::HomeProgramTransition(
                cause,
            ))
        })?;
        let mut state = self.state();
        let medium = state.admitting_medium()?;
        let artifact = RelationalBranchTransitionArtifact::from_positioned(&transition, policy)
            .map_err(|denial| {
                refused(RelationalDurablePublicationRefusal::TransitionEncoding(
                    denial,
                ))
            })?;
        let answer = medium.publish_transition(&artifact);
        state.settle(
            answer,
            RelationalDurabilityUnresolvedSubject::BranchTransition {
                sequence: artifact.sequence(),
                follows: artifact.follows(),
                digest: artifact.payload_digest(),
            },
            || {
                RelationalDurableTransitionPublication::accepted(
                    transition,
                    *artifact.payload_digest().as_bytes(),
                )
            },
        )
    }

    /// Move the medium out so recovery can move it into the rebuilt runtime.
    /// Waits for a publication in flight; afterwards this attachment refuses
    /// every publication. An unresolved fate stays behind: the rebuilt
    /// runtime's recovery has decided it. `None` if it already moved.
    pub fn detach(&self) -> Option<RelationalDurableMediumTransfer> {
        let mut state = self.state();
        // The transfer takes both owners in I/O-then-phase order.
        let mut phase = self.home_phase();
        let medium = state.medium.take()?;
        let phase = phase.take().expect("the live medium owns its phase");
        Some(RelationalDurableMediumTransfer { medium, phase })
    }

    fn home_phase(&self) -> MutexGuard<'_, Option<RelationalHomeProgramPhase>> {
        self.home_phase
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn state(&self) -> MutexGuard<'_, BindingState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl BindingState {
    /// The medium, when no earlier fate is unresolved and the medium has not
    /// moved.
    fn admitting_medium(&self) -> Result<&dyn RelationalDurableMedium, RelationalDurableStepStop> {
        if let Some(earlier) = self.unresolved.get() {
            return Err(RelationalDurableStepStop::Unresolved(earlier.clone()));
        }
        self.medium
            .as_deref()
            .ok_or_else(|| refused(RelationalDurablePublicationRefusal::MediumMoved))
    }

    /// Record the medium's answer for `subject`; an unknown fate blocks every
    /// later publication.
    fn settle<Publication: AuthorityMarker>(
        &mut self,
        answer: DurablePublicationFate<String>,
        subject: RelationalDurabilityUnresolvedSubject,
        committed: impl FnOnce() -> Publication,
    ) -> Result<DurablyCommitted<Publication>, RelationalDurableStepStop> {
        match answer {
            DurablePublicationFate::Committed => Ok(DurablyCommitted::from_owner(committed())),
            DurablePublicationFate::ProvenNoEffect { detail } => Err(refused(
                RelationalDurablePublicationRefusal::ProvenNoEffect { detail },
            )),
            DurablePublicationFate::Indeterminate { detail } => Err(
                RelationalDurableStepStop::Unresolved(self.record_unresolved(subject, detail)),
            ),
        }
    }

    fn record_unresolved(
        &self,
        subject: RelationalDurabilityUnresolvedSubject,
        detail: String,
    ) -> RelationalDurabilityUnresolved {
        self.unresolved
            .get_or_init(|| RelationalDurabilityUnresolved::new(subject, detail))
            .clone()
    }
}

fn refused(refusal: RelationalDurablePublicationRefusal) -> RelationalDurableStepStop {
    RelationalDurableStepStop::Refused(refusal)
}
