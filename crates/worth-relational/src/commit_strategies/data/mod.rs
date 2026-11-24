pub(crate) mod artifacts;
pub(crate) mod canonical_digest;
pub(crate) mod descriptor;
pub(crate) mod execution;
mod execution_failure;
pub(crate) mod lowering;
pub(crate) mod native_codec;
mod native_strategy_intent_scope;
pub(crate) mod registration;
pub(crate) mod request;
mod strategy_aspect_read_record;
pub(crate) mod strategy_id;
pub(crate) mod validation;

pub(crate) use canonical_digest::commit_strategy_registry_digest;

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
pub(crate) mod artifact_basis;
pub(crate) mod lowering_meaning;
pub use lowering_meaning::{
    StrategyLoweringError, StrategyLoweringProvenance, StrategyLoweringSummary,
};
pub(crate) use native_codec::*;
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
