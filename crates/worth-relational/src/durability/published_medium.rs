//! Publication and recovery against the runtime's original durable medium.
use super::medium::{
    home_write_denial::RelationalHomeProgramWriteDenial, source_retirement,
    CheckpointSelectionObservation, DurablePublicationFate, OpaqueCapturedCheckpointSubject,
    RelationalDurabilityUnresolved, RelationalDurabilityUnresolvedSubject,
    RelationalDurableColdSource, RelationalDurableOwnedReadRefusal,
    RelationalDurablePublicationRefusal, RelationalDurableReadBounds,
    RelationalDurableReadProtection, RelationalDurableStepStop,
    RelationalDurableTransitionReadFailure, RelationalMediumRetirementDenial,
    RelationalMediumRetirementProgress,
};
mod binding;
mod commit_publication;
mod preparation_admission;
mod recovery_source;
mod transition_history;
pub use binding::RelationalInitialInputsPublicationStop;
pub(crate) use binding::{
    CheckpointPublication, RelationalCommittedInitialInputs, RelationalDurableCommitReadmission,
    RelationalDurableMediumBinding, RelationalDurableMediumTransfer, SealedRelationalMediumBinding,
};
pub(crate) use commit_publication::RelationalCommitPublication;
pub(crate) use preparation_admission::RelationalDurablePreparationAdmission;
pub use recovery_source::{RelationalDurableCheckpointSource, RelationalDurableRecoverySource};
pub(crate) use transition_history::RelationalTransitionHistoryReader;

use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;

/// Durable storage for Relational's canonical commit history and its branch
/// lifecycle transitions.
///
/// Implementations publish each commit artifact at exactly its position,
/// always one past the last position the medium made durable for this
/// runtime, and each transition artifact at exactly its sequence, always one
/// past the last durable sequence. The two streams are independent: a
/// transition never consumes a position. Publications are issued one at a
/// time across both.
///
/// A medium is moved into exactly one runtime, as a
/// `Box<dyn RelationalDurableMedium>`, by
/// [`RelationalRuntimeBuilder::build_durable`](crate::facade::runtime::RelationalRuntimeBuilder::build_durable).
/// Publication access stays with that owner: recovery moves it into the rebuilt
/// runtime, and a forked runtime never inherits publication access. Speculative
/// forks may carry only a read-only historical-transition reader.
pub trait RelationalDurableMedium: Send + Sync + std::fmt::Debug {
    /// Only an Application Home composition binds its original transition
    /// receiver. A medium without that owner admits no Home transition proof.
    fn home_transition_authority(&self) -> Option<&worth_proof::HomeProgramTransitionAuthority> {
        None
    }
    /// Only the actual Home medium carries its already source-bound selection.
    fn home_world_selection(
        &self,
    ) -> Option<
        std::sync::Arc<
            worth_proof::VerifiedHomeWorldSelection<
                std::sync::Arc<crate::durability::home_world_catalog::RelationalWorldSourceCatalog>,
            >,
        >,
    > {
        None
    }
    fn home_transition_required(&self) -> Option<&worth_proof::HomeProgramTransitionRequired> {
        None
    }
    fn publish_initial_inputs(
        &self,
        _artifact: &crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact,
        _request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> DurablePublicationFate<
        crate::durability::source_initial_inputs::RelationalInitialInputsPublicationDetail,
    > {
        DurablePublicationFate::ProvenNoEffect { detail: crate::durability::source_initial_inputs::RelationalInitialInputsPublicationDetail::UnsupportedProvider }
    }
    /// Non-consuming validation of this original provider's retirement path.
    /// Unknown providers refuse before the Native publication binding is sealed.
    fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalMediumRetirementDenial> {
        source_retirement::unsupported(request, source)
    }
    /// Retire this provider's actual physical capabilities. The original Box
    /// and its inert allocations stay owned until explicit unbudgeted reclaim.
    fn advance_source_retirement(
        &mut self,
        source: &dyn RelationalDurableColdSource,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalMediumRetirementProgress, RelationalMediumRetirementDenial> {
        source_retirement::unsupported(request, source)?;
        unreachable!("unknown provider refuses")
    }

    /// Read the exact Home attempt's actual terminal receipt. A caller-supplied
    /// fate or separately paired receipt is not an answer to this question.
    fn checkpoint_selection(
        &self,
        _subject: &OpaqueCapturedCheckpointSubject,
    ) -> CheckpointSelectionObservation {
        CheckpointSelectionObservation::NotSelected
    }
    /// The same durable journal's read-only source, available before the first
    /// publication. Each committed artifact remains readable at its position;
    /// member placement and retention stay owned by the source provider.
    fn cold_source(&self) -> std::sync::Arc<dyn RelationalDurableColdSource>;

    /// Read one historical transition for explicit replay. Implementations
    /// bound the admitted artifact size and verify its frame before returning.
    fn read_transition(
        &self,
        sequence: crate::branch::transition::sequence::RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure>;
    /// Make one commit artifact durable, or say exactly why not.
    ///
    /// `Committed` means the artifact is durable at its position, including an
    /// identical retry of an artifact that already was. `ProvenNoEffect` means
    /// nothing durable changed. `Indeterminate` means the publication may or
    /// may not have become durable; only the medium's own recovery can tell.
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String>;

    /// Make one branch transition artifact durable at its sequence, with the
    /// same answers as [`Self::publish`].
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String>;
}

pub(crate) mod live_transition;
