pub mod artifacts;
pub mod canonical_digest;
pub mod descriptor;
pub mod execution;
pub mod execution_failure;
pub mod native_codec;
pub mod native_strategy_intent_scope;
pub mod registration;
pub mod request;
pub mod strategy_aspect_read_record;
pub mod strategy_id;
pub mod validation;

pub use canonical_digest::commit_strategy_registry_digest;

pub use artifacts::{
    StrategyCommitArtifactBundle, StrategyIntentScopeDigest, StrategyMergeConflictClass,
    StrategyMergeDescriptor, StrategyMergeSemantics, StrategyReplayDescriptor,
};
pub use descriptor::{
    CommitStrategyDescriptor, CommitStrategyDescriptorDigest, CommitStrategyFamilyName,
    CommitStrategySemanticName, CommitStrategyVersion, PersistentArtifactName,
    StrategyInputSchemaName, StrategyInputSchemaVersion, StrategyIntentName,
    StrategyOutputSchemaName, StrategyPacketContract, StrategyReadContract, StrategyReadCostClass,
    StrategyReadLocalityClass, StrategyReadScopeClass, StrategyTraversalBasis,
};
pub use execution::{
    CanonicalStrategyOutputArtifact, CanonicalStrategyOutputDigest, StrategyExecutionDraft,
    StrategyExecutionResult, StrategyExecutionSummary, StrategyMutationProgram,
    StrategyMutationProgramDigest,
};
pub use execution_failure::{
    StrategyExecutorFailure, StrategyExecutorFailureClass, StrategyExecutorFailureEvidence,
};
pub mod artifact_basis;
pub mod lowering_meaning;
pub use lowering_meaning::{
    StrategyLoweringError, StrategyLoweringProvenance, StrategyLoweringSummary,
};
pub use native_codec::*;
pub use registration::{CommitStrategyRegistration, CommitStrategyRegistrationError};
pub use request::{
    CanonicalStrategyCommitRequest, CanonicalStrategyInputArtifact, CanonicalStrategyInputDigest,
    NativeStrategyCommitRequest, StrategyCallerProvenance, StrategyCommitRequestError,
    StrategyRequestOrigin,
};
pub use strategy_aspect_read_record::{
    StrategyEntityAspectReadRecord, StrategyProjectedAspectReadSet,
};
pub use strategy_id::CommitStrategyId;
pub use validation::StrategyPreviewValidationCostSummary;
