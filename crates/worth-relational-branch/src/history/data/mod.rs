pub mod aspect_history;
pub mod association_discovery;
pub use association_discovery::RelationalAssociationDiscovery;
pub use worth_relational_contracts::internal_port::history::data::recorded_json_allocation;
pub mod replay_retention_denial;
pub use recorded_json_allocation::RecordedRelationalJsonHeapQuote;
pub use replay_retention_denial::RelationalReplayRetentionDenial;
pub use worth_relational_contracts::internal_port::history::data::branch_creation;
pub use worth_relational_contracts::internal_port::history::data::replay_version;
pub mod caller_association;
pub mod canonical_commit_allocation;
pub mod canonical_commit_envelope;
pub use caller_association::{RelationalCallerAssociation, RelationalCallerAssociationRefusal};
pub use canonical_commit_envelope::{validate_authored_restoration_candidate, OutputOriginHistory};
pub mod codec_retirement_closure;
pub use codec_retirement_closure::{historical_producer_antecedent, source_scope_closure};
pub mod codec_retirement_preservation;
pub use canonical_commit_envelope::{
    CanonicalCoupledAuthoredPublication, CanonicalCoupledAuthoredSources,
    CanonicalHistoricalOutputSelection,
};
pub use codec_retirement_closure::select_owned_scope_dependent;
pub use codec_retirement_closure::OwnedScopeDependentSelection;
pub use codec_retirement_closure::{
    capture_materialization_producers, derive_codec_retirement_closure,
    output_materialization_write_matches, prepare_materialization_continuity,
    readmit_required_output_route, select_authored_retirement,
    validate_codec_output_correspondence, CodecRetirementClosureStop, OwnedCodecRetirementClosure,
    SelectedOutputPublication,
};
pub use codec_retirement_preservation::validate_retired_codec_closure;
pub use worth_relational_contracts::internal_port::history::data::canonical_record_allocation;
pub mod committed_version;
pub mod positioned_canonical_commit;
pub mod publication_relations;
pub mod required_output_positions;
pub use required_output_positions::presence as current_route_presence;
pub use required_output_positions::{
    RequiredOutputPublicationPositions, RequiredOutputRoutePosition,
};

use serde::{Deserialize, Serialize};

pub use aspect_history::{
    AspectHistoryCommitSpan, AspectHistoryDigest, AspectHistoryEntry,
    AspectHistoryLineageEventSpan, AspectHistoryOrigin, AspectHistoryQueryResult,
    AspectHistoryResolutionTrace, AspectResolutionContext, HistoryAspectQueryTarget,
    LineageAspectHistory, LineageAspectHistoryQueryResult, LineageAspectResolutionDigest,
};
pub use branch_creation::{BranchCreateError, BranchCreateErrorClass};
pub use canonical_commit_envelope::EmbeddedCheckpointEnvelopeRef;
pub use canonical_commit_envelope::{
    AuthoritativeArtifactEnvelopeRef, CanonicalAuthoredIndexWrite, CanonicalAuthoredIndexWrites,
    CanonicalIndexDefinitionDelta,
};
pub use canonical_commit_envelope::{
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
pub use canonical_commit_envelope::{
    CanonicalCommitEnvelopeBody, CanonicalCommitEnvelopeDraft, CanonicalEnvelopeReadmissionMemory,
    CanonicalOutputPublication,
};
pub use canonical_record_allocation::RecordAllocationClass;
pub use canonical_record_allocation::{
    CancelledRecordAllocation, CanonicalRecordAllocation, RecordAllocationOrigin,
};
pub use committed_version::CommittedVersionSummary;
pub use positioned_canonical_commit::PositionedCanonicalCommit;

pub use coordinates::{BranchId, CommitId, HistoryRetentionClass, VersionGraphPolicy};
pub use worth_relational_contracts::internal_port::history::data::coordinates;

pub use commit_receipt::{OrderedParentList, RelationalCommitReceipt};
pub use history_policy::{HistoryDriftClass, HistoryShapeClassification};
pub use worth_relational_contracts::internal_port::history::data::commit_receipt;
pub use worth_relational_contracts::internal_port::history::data::history_policy;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionNode {
    pub commit: RelationalCommitReceipt,
}

pub use canonical_commit_envelope::{CanonicalEnvelopeSeed, CanonicalEnvelopeWire};

pub use canonical_commit_envelope::{
    CodecDeclarationsReadmissionMemory, StrategyReadmissionMemory,
};

pub use canonical_commit_envelope::EnvelopeProjection;

pub use canonical_commit_envelope::CanonicalEnvelopeControlAllocation;
pub use canonical_commit_envelope::LiveManagedSourceHistory;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub mod inline_type_budget;
