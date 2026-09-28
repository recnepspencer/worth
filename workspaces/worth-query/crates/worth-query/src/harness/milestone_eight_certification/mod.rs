#[cfg(test)]
mod tests;

use crate::application::WorthQueryApplicationFacade;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, GuidedAuthoringPath,
    OrderingSelector, RootEntityKey, WorthQueryPredicateOperand,
};
use crate::basis::{
    resolve_snapshot_basis, BasisAuthorityFamily, BasisResolutionMode, ExecutionBasisIntent,
    ResolvedSnapshotIdentity, SnapshotLineageClass,
};
use crate::composition::{
    GuidedCompositionPath, QueryScopeDescriptor, QueryTemplateDescriptor, TemplateBindingSet,
    TemplateParameterSlot,
};
use crate::harness::certification::{
    digest_parts, CanonicalCertificationRow, CertificationMatrix, HostileExpectation, ParityAnchor,
    RejectionCertificationRow,
};
use crate::identity::{BasisDigest, CanonicalQueryDigest};
use crate::identity_evolution::{
    admit_identity_evolution_query_for_scenario, execute_admitted_identity_evolution_query,
    CorrespondenceIdentityComparison, IdentityEvolutionCertificationResultEvidence,
    IdentityEvolutionComparisonBasisFamily, IdentityEvolutionQueryContext,
    IdentityEvolutionSyntheticScenario, InspectorIdentityArtifact, InspectorIdentityClassification,
    LineageTraversalDescriptor,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::saved_query::{
    evaluate_saved_query_reuse, freeze_composed_saved_query, freeze_direct_saved_query,
    SavedQueryFreezeContext, SavedQueryPersistenceClaim, SavedQueryReuseDescriptor,
    SavedQueryReuseOutcome,
};
use crate::view_shape::{
    admit_view_shape, plan_admitted_view_shape, validate_canonical_bundle_for_admitted_view_shape,
    ViewShapeDescriptor,
};
use crate::view_shape_live::{
    admit_grouped_live_view, execute_grouped_live_view_shape_change,
    execute_live_view_shape_change, lower_view_shape_plan_to_live,
    materialize_authoritative_grouped_baseline,
    materialize_grouped_execution_surface_from_truth_view,
};
use worth_foundational::facade::{
    AspectKey, AspectLocator, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
    ScalarAspectType,
};
use worth_runtime_bridge::facade::{
    encode_snapshot_aspect_read_value, materialize_relational_authoritative_row_set,
    project_relational_grouped_truth, RelationalGroupedProjectionContract,
};
use worth_runtime_bridge::facade::{
    materialize_bridge_grouped_truth_view_from_projection, materialize_bridge_row_set,
    AspectKeySelector, BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity,
    BridgeCommittedPatchItem, BridgeCommittedPatchTarget, BridgeDeliveryReceipt,
    BridgeGroupedTruthViewArtifact, BridgeMappingId, BridgeMappingRegistration,
    BridgeRuntimePolicy, BridgeSignalInvalidationDelivery, BridgeSnapshotReadError,
    BridgeSourceAdapter, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeTruthViewSelector, CoarseRoutingMode, CommittedPatchSource, InvalidationSink,
    MappingSelector, RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
    RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridge,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract,
    SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadRequest,
    SnapshotReadSource, SourceDeclaration, SourceDeclarationIdentity, TruthBranchHeadSource,
    TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity, TruthPatchScope,
    TruthPatchTargetSelector, TruthSnapshotIdentity, TruthSnapshotReader,
};

mod authoring_world;
mod bridge;
mod bundles;
mod classifications;
mod digests;
mod grouped_world;
mod matrix;
mod requirements;
mod rows;
mod scenarios;

use authoring_world::*;
use bridge::*;
use bundles::*;
pub use classifications::MilestoneEightFailureClass;
use classifications::MilestoneEightPerturbationClass;
use digests::*;
use grouped_world::*;
pub use matrix::MilestoneEightCertificationAdapter;
pub use requirements::{
    MILESTONE_EIGHT_REQUIRED_CANONICAL_ROW_NAMES, MILESTONE_EIGHT_REQUIRED_REJECTION_ROW_NAMES,
};
use rows::*;
use scenarios::*;
