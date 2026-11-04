use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::data::RelationalNativeCheckpoint;
use crate::publication::patch::data::PatchStreamPosition;

/// The durable checkpoint a medium selected, with the coverage its owner
/// recorded when it was published: the last commit position and the last
/// branch transition sequence it includes.
///
/// Recovery requires the checkpoint's own coverage to equal both, so a
/// checkpoint filed under the wrong coverage is refused, never reinterpreted.
#[derive(Debug, Clone)]
pub struct RelationalDurableCheckpointSource {
    admission_image:
        Option<crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact>,
    complete: Option<crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact>,
    covered: PatchStreamPosition,
    covered_transitions: RelationalBranchTransitionSequence,
    checkpoint: RelationalNativeCheckpoint,
}

impl RelationalDurableCheckpointSource {
    pub fn new(
        covered: PatchStreamPosition,
        covered_transitions: RelationalBranchTransitionSequence,
        checkpoint: RelationalNativeCheckpoint,
    ) -> Self {
        Self {
            admission_image: None,
            complete: None,
            covered,
            covered_transitions,
            checkpoint,
        }
    }

    pub fn from_complete_artifact(
        artifact: crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact,
    ) -> Result<Self, crate::durability::source_initial_inputs::RelationalInitialInputsRefusal>
    {
        Ok(Self {
            admission_image: None,
            covered: artifact.covered(),
            covered_transitions: artifact.covered_transitions(),
            checkpoint: artifact.checkpoint()?,
            complete: Some(artifact),
        })
    }
    pub(crate) fn complete_artifact(
        &self,
    ) -> Option<&crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact> {
        self.complete.as_ref()
    }

    pub fn from_checkpoint_admission_image(
        artifact: crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact,
        image: crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal>
    {
        use crate::durability::checkpoint_admission::{
            CheckpointAdmissionBindingMismatch as Mismatch,
            RelationalCheckpointAdmissionRefusal as Refusal,
        };
        let mut source =
            Self::from_complete_artifact(artifact).map_err(|_| Refusal::InvalidProjectedMeaning)?;
        let cut = image.cut();
        let worth_foundational::facade::CheckpointAdmissionScope::Home {
            native_origin,
            complete_source_digest,
            ..
        } = cut.scope()
        else {
            return Err(Refusal::BindingMismatch(Mismatch::Scope));
        };
        let complete = source
            .complete
            .as_ref()
            .expect("complete constructor retains its artifact");
        if *native_origin != complete.initial_coordinate().1 {
            return Err(Refusal::BindingMismatch(Mismatch::NativeOrigin));
        }
        let hash =
            |bytes: &[u8]| crate::durability::checkpoint_admission::binding_digest(bytes, request);
        if complete_source_digest != &hash(complete.as_bytes())? {
            return Err(Refusal::BindingMismatch(Mismatch::Source));
        }
        if cut.checkpoint_digest() != &hash(source.checkpoint.bytes())? {
            return Err(Refusal::BindingMismatch(Mismatch::Checkpoint));
        }
        if cut.canonical_through() != source.covered.0
            || cut.transitions_through() != source.covered_transitions.0
        {
            return Err(Refusal::BindingMismatch(Mismatch::Cut));
        }
        source.admission_image = Some(image);
        Ok(source)
    }

    pub(crate) fn admission_image(
        &self,
    ) -> Option<&crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact>
    {
        self.admission_image.as_ref()
    }

    pub fn covered(&self) -> PatchStreamPosition {
        self.covered
    }

    pub fn covered_transitions(&self) -> RelationalBranchTransitionSequence {
        self.covered_transitions
    }

    pub fn checkpoint(&self) -> &RelationalNativeCheckpoint {
        &self.checkpoint
    }
}

/// Everything a medium recovered for one Relational home, as untrusted bytes.
///
/// `tail` holds every durable commit position after the checkpoint's
/// coverage (or from position one without a checkpoint), in ascending order.
/// `transitions` holds every durable branch transition after the
/// checkpoint's transition coverage (or from sequence one), in ascending
/// order. Relational readmits all of it before any of it becomes authority.
#[derive(Debug)]
pub struct RelationalDurableRecoverySource {
    readmitted_originals: Option<
        std::sync::Arc<
            crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs,
        >,
    >,
    initial_inputs: Option<worth_execution::ExecutionImmutableBytes>,
    checkpoint: Option<RelationalDurableCheckpointSource>,
    tail: Vec<RelationalCanonicalCommitArtifact>,
    transitions: Vec<RelationalBranchTransitionArtifact>,
    cold_source: std::sync::Arc<dyn super::RelationalDurableColdSource>,
    source_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RelationalDurableRecoverySource {
    pub fn new(
        checkpoint: Option<RelationalDurableCheckpointSource>,
        tail: Vec<RelationalCanonicalCommitArtifact>,
        transitions: Vec<RelationalBranchTransitionArtifact>,
        cold_source: std::sync::Arc<dyn super::RelationalDurableColdSource>,
    ) -> Self {
        Self {
            readmitted_originals: None,
            initial_inputs: None,
            checkpoint,
            tail,
            transitions,
            cold_source,
            source_memory: None,
        }
    }

    pub fn checkpoint(&self) -> Option<&RelationalDurableCheckpointSource> {
        self.checkpoint.as_ref()
    }

    pub fn tail(&self) -> &[RelationalCanonicalCommitArtifact] {
        &self.tail
    }

    pub fn transitions(&self) -> &[RelationalBranchTransitionArtifact] {
        &self.transitions
    }

    pub(crate) fn into_parts(self) -> RecoverySourceParts {
        RecoverySourceParts {
            readmitted_originals: self.readmitted_originals,
            initial_inputs: self.initial_inputs,
            checkpoint: self.checkpoint,
            tail: self.tail,
            transitions: self.transitions,
            cold_source: self.cold_source,
            _source_memory: self.source_memory,
        }
    }
}

/// A recovery source taken apart for readmission.
pub(crate) struct RecoverySourceParts {
    pub(crate) readmitted_originals: Option<
        std::sync::Arc<
            crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs,
        >,
    >,
    pub(crate) initial_inputs: Option<worth_execution::ExecutionImmutableBytes>,
    pub(crate) checkpoint: Option<RelationalDurableCheckpointSource>,
    pub(crate) tail: Vec<RelationalCanonicalCommitArtifact>,
    pub(crate) transitions: Vec<RelationalBranchTransitionArtifact>,
    pub(crate) cold_source: std::sync::Arc<dyn super::RelationalDurableColdSource>,
    pub(crate) _source_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

mod home_preflight;
mod owned;
