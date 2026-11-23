mod aspect_history;
mod association_discovery;
pub use association_discovery::RelationalAssociationDiscovery;
pub(crate) mod recorded_json_allocation;
mod replay_retention_denial;
pub(crate) mod replay_version;
pub use recorded_json_allocation::RecordedRelationalJsonHeapQuote;
pub use replay_retention_denial::RelationalReplayRetentionDenial;
pub(crate) mod branch_creation;
mod caller_association;
mod canonical_commit_allocation;
pub(crate) mod canonical_commit_envelope;
pub use caller_association::{RelationalCallerAssociation, RelationalCallerAssociationRefusal};
pub(crate) use canonical_commit_envelope::{
    validate_authored_restoration_candidate, OutputOriginHistory,
};
mod codec_retirement_closure;
pub(crate) use codec_retirement_closure::{historical_producer_antecedent, source_scope_closure};
mod codec_retirement_preservation;
pub(crate) use canonical_commit_envelope::{
    CanonicalCoupledAuthoredPublication, CanonicalCoupledAuthoredSources,
    CanonicalHistoricalOutputSelection,
};
pub(crate) use codec_retirement_closure::select_owned_scope_dependent;
pub(crate) use codec_retirement_closure::OwnedScopeDependentSelection;
pub(crate) use codec_retirement_closure::{
    capture_materialization_producers, derive_codec_retirement_closure,
    output_materialization_write_matches, prepare_materialization_continuity,
    readmit_required_output_route, select_authored_retirement, select_authored_source_retirement,
    validate_codec_output_correspondence, AuthoredRetirementSelection, CodecRetirementClosureStop,
    OwnedCodecRetirementClosure, SelectedAuthoredRetirement, SelectedOutputPublication,
};
pub(crate) use codec_retirement_preservation::validate_retired_codec_closure;
pub(crate) mod canonical_record_allocation;
mod committed_version;
pub(crate) mod merge_branch_basis;
#[cfg(test)]
mod merge_branch_basis_foundational;
mod positioned_canonical_commit;
mod publication_relations;
pub(crate) mod record_allocation_class;
mod required_output_positions;
pub(crate) use required_output_positions::presence as current_route_presence;
pub(crate) use required_output_positions::{
    RequiredOutputPublicationPositions, RequiredOutputRoutePosition,
};

use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, RelationId};

pub use aspect_history::{
    AspectHistoryCommitSpan, AspectHistoryDigest, AspectHistoryEntry,
    AspectHistoryLineageEventSpan, AspectHistoryOrigin, AspectHistoryQueryResult,
    AspectHistoryResolutionTrace, AspectResolutionContext, HistoryAspectQueryTarget,
    LineageAspectHistory, LineageAspectHistoryQueryResult, LineageAspectResolutionDigest,
};
pub use branch_creation::{BranchCreateError, BranchCreateErrorClass};
#[cfg(test)]
pub(crate) use canonical_commit_envelope::EmbeddedCheckpointEnvelopeRef;
pub(crate) use canonical_commit_envelope::{
    AuthoritativeArtifactEnvelopeRef, CanonicalAuthoredIndexWrite, CanonicalAuthoredIndexWrites,
    CanonicalIndexDefinitionDelta,
};
pub(crate) use canonical_commit_envelope::{
    CanonicalAuthoredRestoration, CanonicalAuthoredRetirement, CanonicalCodecRetirementKind,
    CanonicalOutputMember, CanonicalProducerRetirement, CanonicalRequiredOutputCodecRetirement,
    CanonicalRequiredOutputMaterialization, CanonicalRequiredOutputMaterializationRecord,
    CanonicalRequiredOutputOrigin, CanonicalRequiredOutputRelation,
};
pub use canonical_commit_envelope::{
    CanonicalCommitAuthorityKind, CanonicalCommitEnvelope, CanonicalOwnedGraphSelection,
    CanonicalRetainedEntitySelection, RelationalDescriptiveTouch, RelationalDescriptiveTouchGraph,
    RelationalDescriptiveTouchPrecision, RelationalIntroducedSchemaObservation,
    RelationalOriginalInputObservation, RelationalReplayRecord, RelationalTouchAdjacencyDirection,
    ReplaySchemaVersion, SharedCanonicalCommitEnvelope,
};
pub(crate) use canonical_commit_envelope::{
    CanonicalCommitEnvelopeBody, CanonicalCommitEnvelopeDraft, CanonicalEnvelopeReadmissionMemory,
    CanonicalOutputPublication,
};
pub use canonical_record_allocation::RecordAllocationClass;
pub(crate) use canonical_record_allocation::{CanonicalRecordAllocation, RecordAllocationOrigin};
pub use committed_version::CommittedVersionSummary;
pub use merge_branch_basis::{
    MergeBaseSelectionRule, MergeBasisAncestors, RelationalMergeBranchBasis,
    RelationalMergeBranchBasisDenial, ResolvedMergeBase,
};
#[cfg(test)]
pub use merge_branch_basis_foundational::{
    RelationalFoundationalCurrentMergeBranchBasisArtifact,
    RelationalMergeBranchBasisFoundationalLoweringDenial,
};
pub(crate) use positioned_canonical_commit::PositionedCanonicalCommit;

pub(crate) mod coordinates;
pub use coordinates::{BranchId, CommitId, HistoryRetentionClass, VersionGraphPolicy};

pub(crate) mod commit_receipt;
pub(crate) mod history_policy;
pub use commit_receipt::{OrderedParentList, RelationalCommitReceipt};
pub use history_policy::{HistoryDriftClass, HistoryShapeClassification};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct VersionNode {
    pub(crate) commit: RelationalCommitReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MergeConflictRecord {
    Entity(EntityId),
    Relation(RelationId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeInspection {
    pub source_branch: BranchId,
    pub target_branch: BranchId,
    pub source_head: Option<RelationalCommitReceipt>,
    pub target_head: Option<RelationalCommitReceipt>,
    /// Current merge-base result under the runtime's common-ancestor selection
    /// rule.
    pub merge_base: Option<CommitId>,
    /// Branch-local ancestor closure for the source head after removing the
    /// merge-base ancestor closure, ordered by ascending `CommitId`.
    pub source_only_commits: Vec<CommitId>,
    /// Branch-local ancestor closure for the target head after removing the
    /// merge-base ancestor closure, ordered by ascending `CommitId`.
    pub target_only_commits: Vec<CommitId>,
    pub conflicting_records: Vec<MergeConflictRecord>,
    pub can_merge: bool,
}

pub(crate) use canonical_commit_envelope::{CanonicalEnvelopeSeed, CanonicalEnvelopeWire};

pub(crate) use canonical_commit_envelope::{
    CodecDeclarationsReadmissionMemory, StrategyReadmissionMemory,
};

pub(crate) use canonical_commit_envelope::EnvelopeProjection;

pub(crate) use canonical_commit_envelope::CanonicalEnvelopeControlAllocation;
pub(crate) use canonical_commit_envelope::LiveManagedSourceHistory;
pub(crate) use canonical_commit_envelope::{AppliedPublicationInputAccess, LivePublicationInputs};
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
